use std::io::Error;

use uuid::Uuid;
use internal::discovery::Discovery;
use internal::operations;
use internal::package::Pkg;
use types;

pub(crate) enum Msg {
    Start,
    Shutdown,
    Tick,
    Establish(Discovery, types::Endpoint),
    Established(Uuid),
    Arrived(Pkg),
    ConnectionClosed(Uuid, Error),
    DiscoveryError(Discovery, Error),
    NewOp(operations::OperationWrapper),
    Send(Pkg),
    Marker, // Use as checkpoint detection.
}

impl Msg {
    pub(crate) fn new_op(op: operations::OperationWrapper) -> Msg {
        Msg::NewOp(op)
    }
}
