use std::collections::HashMap;
use std::io;

use bytes::BytesMut;
use protobuf;
use uuid::Uuid;

use internal::command::Cmd;
use internal::connection::Connection;
use internal::messages;
use internal::operations::{ OperationError, OperationWrapper, OperationId, Tracking };
use internal::package::Pkg;
use types;

struct Request {
    session: OperationId,
    cmd: Cmd,
}

impl Request {
    fn new(session: OperationId, cmd: Cmd) -> Request {
        Request {
            session,
            cmd,
        }
    }
}

struct Requests {
    sessions: HashMap<OperationId, OperationWrapper>,
    assocs: HashMap<Uuid, Request>,
    buffer: BytesMut,
}

impl Requests {
    fn new() -> Requests {
        Requests {
            sessions: HashMap::new(),
            assocs: HashMap::new(),
            buffer: BytesMut::new(),
        }
    }

    fn register(&mut self, conn: &Connection, mut op: OperationWrapper) {
        let session_id = op.id;
        let success = {

            match op.send(conn.id, &mut self.buffer).map(|out| out.produced_pkgs()) {
                Ok(pkgs) => {

                    for pkg in pkgs.as_slice() {
                        self.assocs.insert(pkg.correlation, Request::new(session_id, pkg.cmd));
                    }

                    conn.enqueue_all(pkgs);

                    true
                },

                Err(e) => {
                    error!("Exception occured when issuing requests: {}", e);

                    false
                },
            }
        };

        if success {
            self.sessions.insert(op.id, op);
        }
    }

    fn handle_pkg(&mut self, awaiting: &mut Vec<OperationWrapper>, conn: &Connection, pkg: Pkg)
        -> Option<types::Endpoint>
    {
        enum Out {
            Failed,
            Handled(Option<types::Endpoint>),
        }

        struct Outcome {
            session_requests: Option<Vec<Uuid>>,
            force_reconnect: Option<types::Endpoint>,
        }

        let pkg_id  = pkg.correlation;
        let pkg_cmd = pkg.cmd;

        if let Some(req) = self.assocs.remove(&pkg_id) {
            let session_id = req.session;
            let original_cmd = req.cmd;

            debug!("Package [{}]: command {:?} received {:?}.", pkg_id, original_cmd, pkg_cmd);

            let outcome = {
                let mut op =
                    self.sessions
                        .remove(&session_id)
                        .expect(&format!("Unknown session!: Package[{}]: command {:?}, received {:?}.", pkg_id, original_cmd, pkg_cmd));

                let out = {

                    match pkg.cmd {
                        Cmd::BadRequest => {
                            let msg = pkg.build_text();

                            error!("Bad request for command {:?}: {}.", original_cmd, msg);

                            op.failed(OperationError::ServerError(Some(msg)));

                            Out::Failed
                        },

                        Cmd::NotAuthenticated => {
                            error!("Not authenticated for command {:?}.", original_cmd);

                            op.failed(OperationError::AuthenticationRequired);

                            Out::Failed
                        },

                        Cmd::NotHandled => {
                            warn!("Not handled request {:?} id {}.", original_cmd, pkg_id);

                            let msg: io::Result<Option<types::Endpoint>> =
                                pkg.to_message().and_then(|not_handled: messages::NotHandled|
                                {
                                    if let messages::NotHandled_NotHandledReason::NotMaster = not_handled.get_reason() {
                                        let master_info: messages::NotHandled_MasterInfo =
                                            protobuf::parse_from_bytes(not_handled.get_additional_info())?;

                                        // TODO - Support reconnection on the secure port when we are going to
                                        // implement SSL connection.
                                        let addr_str = format!("{}:{}", master_info.get_external_tcp_address(), master_info.get_external_tcp_port());
                                        let external_tcp_port = types::Endpoint::from_addrs(addr_str)?;

                                        Ok(Some(external_tcp_port))
                                    } else {
                                        Ok(None)
                                    }
                                });

                            match msg {
                                Ok(endpoint_opt) => {
                                    match endpoint_opt {
                                        Some(endpoint) => {
                                            warn!("Received a non master error on command {:?} id {}, [{:?}]", original_cmd, pkg_id, endpoint);

                                            awaiting.push(op);

                                            Out::Handled(Some(endpoint))
                                        },

                                        None => {
                                            warn!("The server has either not started or is too busy.
                                                  Retrying command {:?} id {}.", original_cmd, pkg_id);

                                            match op.retry(&mut self.buffer, pkg_id) {
                                                Ok(outcome) => {
                                                    let pkgs = outcome.produced_pkgs();

                                                    if !pkgs.is_empty() {
                                                        for pkg in pkgs.as_slice() {
                                                            self.assocs.insert(pkg.correlation, Request::new(session_id, pkg.cmd));
                                                        }

                                                        self.sessions.insert(session_id, op);
                                                        conn.enqueue_all(pkgs);
                                                    }

                                                    Out::Handled(None)
                                                },

                                                Err(error) => {
                                                    error!(
                                                        "An error occured when retrying command {:?} id {}: {}.",
                                                        original_cmd, pkg_id, error
                                                    );

                                                    Out::Failed
                                                },
                                            }
                                        },
                                    }
                                },

                                Err(error) => {
                                    error!("Decoding error: can't decode NotHandled message: {}.", error);

                                    Out::Failed
                                },
                            }
                        },

                        _ => match op.receive(conn.id, &mut self.buffer, pkg) {
                            Ok(outcome) => {
                                let is_continue = outcome.is_continue();
                                let pkgs = outcome.produced_pkgs();

                                if !pkgs.is_empty() {
                                    for pkg in pkgs.as_slice() {
                                        self.assocs.insert(pkg.correlation, Request::new(session_id, pkg.cmd));
                                    }

                                    self.sessions.insert(session_id, op);
                                    conn.enqueue_all(pkgs);
                                }

                                if is_continue {
                                    self.assocs.insert(pkg_id, req);
                                }

                                Out::Handled(None)
                            },

                            Err(e) => {
                                error!("An error occured when running operation: {}", e);
                                let msg = format!("Exception raised: {}", e);

                                op.failed(OperationError::InvalidOperation(msg));

                                Out::Failed
                            },
                        },
                    }
                };

                match out {
                    Out::Failed => {
                        for req_id in op.request_keys() {
                            self.assocs.remove(req_id);
                        }

                        None
                    },

                    Out::Handled(force_reconnect) => {
                        force_reconnect
                    },
                }
            };

            outcome
        } else {
            warn!("Package [{}] not handled: cmd {:?}.", pkg_id, pkg_cmd);

            None
        }
    }

    fn check_and_retry(&mut self, conn: &Connection) {
        let mut sessions_to_delete = Vec::new();

        for op in self.sessions.values_mut() {

            let result = op.check_and_retry(conn.id, &mut self.buffer);

            match result {
                Ok(outcome) => {
                    if outcome.is_done() {
                        for id in op.request_keys() {
                            self.assocs.remove(id);
                        }

                        sessions_to_delete.push(op.id);
                    } else {
                        let pkgs = outcome.produced_pkgs();

                        if !pkgs.is_empty() {
                            for pkg in pkgs.as_slice() {
                                self.assocs.insert(pkg.correlation, Request::new(op.id, pkg.cmd));
                            }

                            conn.enqueue_all(pkgs);
                        }
                    }
                },

                Err(e) => {
                    error!("Exception raised when checking out operation: {}", e);
                    let msg = format!("Exception raised: {}", e);

                    op.failed(OperationError::InvalidOperation(msg));

                    for id in op.request_keys() {
                        self.assocs.remove(id);
                    }

                    sessions_to_delete.push(op.id);
                },
            }
        }

        for session_id in sessions_to_delete {
            self.sessions.remove(&session_id);
        }
    }

    pub(crate) fn abort(&mut self) {
        for op in self.sessions.values_mut() {
            op.failed(OperationError::Aborted);
        }
    }
}

pub(crate) struct Registry {
    requests: Requests,
    awaiting: Vec<OperationWrapper>,
}

impl Registry {
    pub(crate) fn new() -> Registry {
        Registry {
            requests: Requests::new(),
            awaiting: Vec::new(),
        }
    }

    pub(crate) fn register(&mut self, op: OperationWrapper, conn: Option<&Connection>) {
        match conn {
            None       => self.awaiting.push(op),
            Some(conn) => self.requests.register(conn, op),
        }
    }

    pub(crate) fn handle(&mut self, pkg: Pkg, conn: &Connection) -> Option<types::Endpoint> {
        self.requests.handle_pkg(&mut self.awaiting, conn, pkg)
    }

    pub(crate) fn check_and_retry(&mut self, conn: &Connection) {
        self.requests.check_and_retry(conn);

        while let Some(op) = self.awaiting.pop() {
            self.register(op, Some(conn));
        }
    }

    pub(crate) fn abort(&mut self) {
        self.requests.abort();

        for op in self.awaiting.iter_mut() {
            op.failed(OperationError::Aborted);
        }
    }
}
