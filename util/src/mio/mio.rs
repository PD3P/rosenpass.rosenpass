use std::os::fd::{OwnedFd, RawFd};

use crate::{
    fd::{claim_fd, claim_fd_inplace},
    result::OkExt,
};

/// Module containing I/O interest flags for Unix operations (see also: [mio::Interest])
pub mod interest {
    use mio::Interest;

    /// Interest flag indicating readability
    pub const R: Interest = Interest::READABLE;

    /// Interest flag indicating writability
    pub const W: Interest = Interest::WRITABLE;

    /// Interest flag indicating both readability and writability
    pub const RW: Interest = R.add(W);
}
