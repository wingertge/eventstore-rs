use std::io::Error;

use uuid::Uuid;
use internal::operations;
use internal::package::Pkg;
use types;

pub(crate) enum Msg {
    Start,
    Shutdown,
    Tick,
    Establish(types::Endpoint),
    Established(Uuid),
    Arrived(Pkg),
    ConnectionClosed(Uuid, Error),
    DiscoveryError(Error),
    NewOp(operations::OperationWrapper),
    Send(Pkg),
    Marker, // Use as checkpoint detection.
}

impl Msg {
    pub(crate) fn new_op(op: operations::OperationWrapper) -> Msg {
        Msg::NewOp(op)
    }
}
