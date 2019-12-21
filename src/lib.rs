/// Provides a TCP client for [GetEventStore] datatbase.

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

#[cfg(feature = "es5-and-below")]
mod discovery;
mod internal;
mod connection;
#[cfg(feature = "es5-and-below")]
pub mod types;
pub mod es6;

pub use connection::{
    Connection,
    ConnectionBuilder,
};

#[cfg(feature = "es5-and-below")]
pub use internal::{
    commands,
    // operations::OperationError
};

// pub use types::*;
