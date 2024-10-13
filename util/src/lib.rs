#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![recursion_limit = "256"]

pub mod b64;
pub mod build;
pub mod controlflow;
pub mod fd;
pub mod file;
pub mod functional;
pub mod io;
pub mod length_prefix_encoding;
pub mod mem;
pub mod mio;
pub mod option;
pub mod result;
pub mod time;
pub mod typenum;
pub mod zerocopy;
pub mod zeroize;
