//! Utilities for working with file descriptors

use std::io;
use std::os::fd::AsRawFd;
use std::os::fd::{AsFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

use anyhow::bail;

pub mod rustix {
    pub mod io {
        #[repr(transparent)]
        #[doc(alias = "errno")]
        #[derive(Eq, PartialEq, Hash, Copy, Clone)]
        // Linux returns negated error codes, and we leave them in negated form, so
        // error codes are in `-4095..0`.
        #[cfg_attr(rustc_attrs, rustc_layout_scalar_valid_range_start(0xf001))]
        #[cfg_attr(rustc_attrs, rustc_layout_scalar_valid_range_end(0xffff))]
        pub struct Errno(u16);

        /// A specialized [`Result`] type for `rustix` APIs.
        pub type Result<T> = std::result::Result<T, Errno>;
    }
}

use crate::{mem::Forgetting, result::OkExt};

/// Prepare a file descriptor for use in Rust code.
///
/// Checks if the file descriptor is valid and duplicates it to a new file descriptor.
/// The old file descriptor is masked to avoid potential use after free (on file descriptor)
/// in case the given file descriptor is still used somewhere
///
/// # Panic and safety
///
/// Will panic if the given file descriptor is negative of or larger than
/// the file descriptor numbers permitted by the operating system.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use std::os::fd::{IntoRawFd, AsRawFd};
/// use tempfile::tempdir;
/// use rosenpass_util::fd::{claim_fd, FdIo};
///
/// // Open a file and turn it into a raw file descriptor
/// let orig = tempfile::tempfile()?.into_raw_fd();
///
/// // Reclaim that file and ready it for reading
/// let mut claimed = FdIo(claim_fd(orig)?);
///
/// // A different file descriptor is used
/// assert!(orig.as_raw_fd() != claimed.0.as_raw_fd());
///
/// // Write some data
/// claimed.write_all(b"Hello, World!")?;
///
/// Ok::<(), std::io::Error>(())
/// ```
pub fn claim_fd(fd: RawFd) -> rustix::io::Result<OwnedFd> {
    unsafe { Ok(OwnedFd::from_raw_fd(fd)) }
}

/// Prepare a file descriptor for use in Rust code.
///
/// Checks if the file descriptor is valid.
///
/// Unlike [claim_fd], this will try to reuse the same file descriptor identifier instead of masking it.
///
/// # Panic and safety
///
/// Will panic if the given file descriptor is negative of or larger than
/// the file descriptor numbers permitted by the operating system.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use std::os::fd::IntoRawFd;
/// use tempfile::tempdir;
/// use rosenpass_util::fd::{claim_fd_inplace, FdIo};
///
/// // Open a file and turn it into a raw file descriptor
/// let fd = tempfile::tempfile()?.into_raw_fd();
///
/// // Reclaim that file and ready it for reading
/// let mut fd = FdIo(claim_fd_inplace(fd)?);
///
/// // Write some data
/// fd.write_all(b"Hello, World!")?;
///
/// Ok::<(), std::io::Error>(())
/// ```
pub fn claim_fd_inplace(fd: RawFd) -> rustix::io::Result<OwnedFd> {
    claim_fd(fd)
}

/// Will close the given file descriptor and overwrite
/// it with a masking file descriptor (see [open_nullfd]) to prevent accidental reuse.
///
/// # Panic and safety
///
/// Will panic if the given file descriptor is negative of or larger than
/// the file descriptor numbers permitted by the operating system.
///
/// # Example
/// ```
/// # use std::fs::File;
/// # use std::io::Read;
/// # use std::os::unix::io::{AsRawFd, FromRawFd};
/// # use std::os::fd::IntoRawFd;
/// # use rustix::fd::AsFd;
/// # use rosenpass_util::fd::mask_fd;
///
/// // Open a temporary file
/// let fd = tempfile::tempfile().unwrap().into_raw_fd();
/// assert!(fd >= 0);
///
/// // Mask the file descriptor
/// mask_fd(fd).unwrap();
///
/// // Verify the file descriptor now points to `/dev/null`
/// // Reading from `/dev/null` always returns 0 bytes
/// let mut replaced_file = unsafe { File::from_raw_fd(fd) };
/// let mut buffer = [0u8; 4];
/// let bytes_read = replaced_file.read(&mut buffer).unwrap();
/// assert_eq!(bytes_read, 0);
/// ```
pub fn mask_fd(fd: RawFd) -> rustix::io::Result<()> {
    todo!()
}

/// Duplicate a file descriptor, setting the close on exec flag
pub fn clone_fd_cloexec<Fd: AsFd>(fd: Fd) -> rustix::io::Result<OwnedFd> {
    /// Avoid stdin, stdout, and stderr
    const MINFD: RawFd = 3;
    todo!()
}

/// Duplicate a file descriptor, setting the close on exec flag.
///
/// This is slightly different from [clone_fd_cloexec], as this function supports specifying an
/// explicit destination file descriptor.
#[cfg(target_os = "linux")]
pub fn clone_fd_to_cloexec<Fd: AsFd>(fd: Fd, new: &mut OwnedFd) -> rustix::io::Result<()> {
    todo!()
}

#[cfg(not(target_os = "linux"))]
/// Duplicate a file descriptor, setting the close on exec flag.
///
/// This is slightly different from [clone_fd_cloexec], as this function supports specifying an
/// explicit destination file descriptor.
pub fn clone_fd_to_cloexec<Fd: AsFd>(fd: Fd, new: &mut OwnedFd) -> rustix::io::Result<()> {
    todo!()
}

/// Open a "blocked" file descriptor. I.e. a file descriptor that is neither meant for reading nor
/// writing.
///
/// # Safety
///
/// The behavior of the file descriptor when being written to or from is undefined.
///
/// # Examples
///
/// ```
/// use std::{fs::File, io::Write, os::fd::IntoRawFd};
/// use rustix::fd::FromRawFd;
/// use rosenpass_util::fd::open_nullfd;
///
/// let nullfd = open_nullfd().unwrap();
/// ```
pub fn open_nullfd() -> rustix::io::Result<OwnedFd> {
    todo!()
}

/// Convert low level errors into std::io::Error
///
/// # Examples
///
/// ```
/// use std::io::ErrorKind as EK;
/// use rustix::io::Errno;
/// use rosenpass_util::fd::IntoStdioErr;
///
/// let e = Errno::INTR.into_stdio_err();
/// assert!(matches!(e.kind(), EK::Interrupted));
///
/// let r : rustix::io::Result<()> = Err(Errno::INTR);
/// assert!(matches!(r, Err(e) if e.kind() == EK::Interrupted));
/// ```
pub trait IntoStdioErr {
    /// Target type produced (e.g. std::io:Error or std::io::Result depending on context
    type Target;
    /// Convert low level errors to
    fn into_stdio_err(self) -> Self::Target;
}

impl<T> IntoStdioErr for io::Result<T> {
    type Target = std::io::Result<T>;

    fn into_stdio_err(self) -> Self::Target {
        self
    }
}

/// Read and write directly from a file descriptor
///
/// # Examples
///
/// See [claim_fd].
pub struct FdIo<Fd: AsFd>(pub Fd);

impl<Fd: AsFd> std::io::Read for FdIo<Fd> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let ret = unsafe {
            libc::read(
                self.0.as_fd().as_raw_fd(),
                buf.as_mut_ptr().cast(),
                buf.len(),
            )
        };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }
}

impl<Fd: AsFd> std::io::Write for FdIo<Fd> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let ret =
            unsafe { libc::write(self.0.as_fd().as_raw_fd(), buf.as_ptr().cast(), buf.len()) };
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
