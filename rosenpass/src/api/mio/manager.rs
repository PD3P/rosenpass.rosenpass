use std::{borrow::BorrowMut, io};

use mio::net::{UnixListener, UnixStream};

use rosenpass_util::{
    functional::ApplyExt, io::nonblocking_handle_io_errors, mio::interest::RW as MIO_RW,
};

use crate::app_server::{AppServer, AppServerIoSource};

use super::{MioConnection, MioConnectionContext};

/// This is in essence a unix listener for API connections.
///
/// It contains a number of [UnixListener]s and the associated [MioConnection]s encapsulating [mio::net::UnixListener]s.
#[derive(Default, Debug)]
pub struct MioManager {
    listeners: Vec<UnixListener>,
    connections: Vec<Option<MioConnection>>,
}

/// Points at a particular source of IO events inside [MioManager]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MioManagerIoSource {
    // Source of IO events is the Nth unix socket listener (see [MioManager::listeners])
    Listener(usize),
    // Source of IO events is the Nth unix socket listener (see [MioManager::connections])
    Connection(usize),
}

impl MioManager {
    /// Construct an empty [Self]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Focus in on a particular [MioConnection] inside a [MioManager]
///
/// This is mainly used to implement [MioConnectionContext].
struct MioConnectionFocus<'a, T: ?Sized + MioManagerContext> {
    /// [MioConnectionContext] to access the [MioManager] instance and [AppServer]
    ctx: &'a mut T,
    /// Index of the connection referenced to by [Self]
    conn_idx: usize,
}

impl<'a, T: ?Sized + MioManagerContext> MioConnectionFocus<'a, T> {
    /// Produce a MioConnectionContext from the [MioConnectionContext] and the connection index
    fn new(ctx: &'a mut T, conn_idx: usize) -> Self {
        Self { ctx, conn_idx }
    }
}

pub trait MioManagerContext {
    /// Reference to the [MioManager]
    fn mio_manager(&self) -> &MioManager;
    /// Reference to the [MioManager], mutably
    fn mio_manager_mut(&mut self) -> &mut MioManager;
    /// Reference to the [AppServer] this [MioManager] is associated with
    fn app_server(&self) -> &AppServer;
    /// Mutable reference to the [AppServer] this [MioManager] is associated with
    fn app_server_mut(&mut self) -> &mut AppServer;

    /// Add a new [UnixListener] to listen for API connections on
    fn add_listener(&mut self, mut listener: UnixListener) -> io::Result<()> {
        let srv = self.app_server_mut();
        let mio_token = srv.mio_token_dispenser.dispense();
        srv.mio_poll
            .registry()
            .register(&mut listener, mio_token, MIO_RW)?;
        let io_source = self
            .mio_manager()
            .listeners
            .len()
            .apply(MioManagerIoSource::Listener)
            .apply(AppServerIoSource::MioManager);
        self.mio_manager_mut().listeners.push(listener);
        self.app_server_mut()
            .register_io_source(mio_token, io_source);

        Ok(())
    }

    /// Add a new connection to an API client
    fn add_connection(&mut self, connection: UnixStream) -> io::Result<()> {
        let connection = MioConnection::new(self.app_server_mut(), connection)?;
        let mio_token = connection.mio_token();
        let conns: &mut Vec<Option<MioConnection>> =
            self.mio_manager_mut().connections.borrow_mut();
        let idx = conns
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_some())
            .map(|(idx, _)| idx)
            .unwrap_or(conns.len());
        conns.insert(idx, Some(connection));
        let io_source = idx
            .apply(MioManagerIoSource::Listener)
            .apply(AppServerIoSource::MioManager);
        self.app_server_mut()
            .register_io_source(mio_token, io_source);
        Ok(())
    }

    /// Poll a particular [MioManagerIoSource] in this [MioManager]
    fn poll_particular(&mut self, io_source: MioManagerIoSource) -> anyhow::Result<()> {
        use MioManagerIoSource as S;
        match io_source {
            S::Listener(idx) => self.accept_from(idx)?,
            S::Connection(idx) => self.poll_particular_connection(idx)?,
        };
        Ok(())
    }

    /// Check for new connections and poll all the [MioConnectionContext]s managed by [Self]
    fn poll(&mut self) -> anyhow::Result<()> {
        self.accept_connections()?;
        self.poll_connections()?;
        Ok(())
    }

    /// Check all the [UnixListener]s managed by this [MioManager] for new connections
    fn accept_connections(&mut self) -> io::Result<()> {
        for idx in 0..self.mio_manager_mut().listeners.len() {
            self.accept_from(idx)?;
        }
        Ok(())
    }

    /// Check a particular [UnixListener] managed by this for new connections.
    fn accept_from(&mut self, idx: usize) -> io::Result<()> {
        // Accept connection until the socket would block or returns another error
        // TODO: This currently only adds connections--we eventually need the ability to remove
        // them as well, see the note in connection.rs
        loop {
            match nonblocking_handle_io_errors(|| self.mio_manager().listeners[idx].accept())? {
                None => break,
                Some((conn, _addr)) => {
                    self.add_connection(conn)?;
                }
            };
        }

        Ok(())
    }

    /// Call [MioConnectionContext::poll] on all the [MioConnection]s in This
    fn poll_connections(&mut self) -> anyhow::Result<()> {
        for idx in 0..self.mio_manager().connections.len() {
            self.poll_particular_connection(idx)?;
        }
        Ok(())
    }

    /// Call [MioConnectionContext::poll] on a particular connection
    fn poll_particular_connection(&mut self, idx: usize) -> anyhow::Result<()> {
        if self.mio_manager().connections[idx].is_none() {
            return Ok(());
        }

        let mut conn = MioConnectionFocus::new(self, idx);
        conn.poll()?;

        if conn.should_close() {
            let conn = self.mio_manager_mut().connections[idx].take().unwrap();
            let mio_token = conn.mio_token();
            if let Err(e) = conn.close(self.app_server_mut()) {
                log::warn!("Error while closing API connection {e:?}");
            };
            self.app_server_mut().unregister_io_source(mio_token);
        }

        Ok(())
    }
}

impl<T: ?Sized + MioManagerContext> MioConnectionContext for MioConnectionFocus<'_, T> {
    fn mio_connection(&self) -> &MioConnection {
        self.ctx.mio_manager().connections[self.conn_idx]
            .as_ref()
            .unwrap()
    }

    fn app_server(&self) -> &AppServer {
        self.ctx.app_server()
    }

    fn mio_connection_mut(&mut self) -> &mut MioConnection {
        self.ctx.mio_manager_mut().connections[self.conn_idx]
            .as_mut()
            .unwrap()
    }

    fn app_server_mut(&mut self) -> &mut AppServer {
        self.ctx.app_server_mut()
    }
}
