use std::collections::HashMap;
use std::io;
use std::time::{ Duration, Instant };

use bytes::BytesMut;
use protobuf;
use uuid::Uuid;

use internal::command::Cmd;
use internal::connection::Connection;
use internal::messages;
use internal::operations::{ OperationError, OperationWrapper, OperationId, ImplResult };
use internal::package::Pkg;
use types;

// struct Request {
//     session: OperationId,
//     cmd: Cmd,
// }

// impl Request {
//     fn new(session: OperationId, cmd: Cmd) -> Request {
//         Request {
//             session,
//             cmd,
//         }
//     }
// }

// struct Requests {
//     sessions: HashMap<OperationId, OperationWrapper>,
//     assocs: HashMap<Uuid, Request>,
//     buffer: BytesMut,
// }

// impl Requests {
//     fn new() -> Requests {
//         Requests {
//             sessions: HashMap::new(),
//             assocs: HashMap::new(),
//             buffer: BytesMut::new(),
//         }
//     }

//     fn register(&mut self, conn: &Connection, mut op: OperationWrapper) {
//         let session_id = op.id;
//         let success = {

//             match op.send(conn.id, &mut self.buffer).map(|out| out.produced_pkgs()) {
//                 Ok(pkgs) => {

//                     for pkg in pkgs.as_slice() {
//                         self.assocs.insert(pkg.correlation, Request::new(session_id, pkg.cmd));
//                     }

//                     conn.enqueue_all(pkgs);

//                     true
//                 },

//                 Err(e) => {
//                     error!("Exception occured when issuing requests: {}", e);

//                     false
//                 },
//             }
//         };

//         if success {
//             self.sessions.insert(op.id, op);
//         }
//     }

//     fn handle_pkg(&mut self, awaiting: &mut Vec<OperationWrapper>, conn: &Connection, pkg: Pkg)
//         -> Option<types::Endpoint>
//     {
//         struct Completed {
//             session: OperationWrapper,
//         }

//         struct Continue {
//             session: OperationWrapper,
//             packages: Vec<Pkg>,
//         }

//         struct ForceReconnect {
//             session: OperationWrapper,
//             new_endpoint: types::Endpoint,
//         }

//         struct Response {
//             session: OperationWrapper,
//             original_cmd: Cmd,
//         }

//         enum Out {
//             Completed(Completed),
//             Continue(Continue),
//             ForceReconnect(ForceReconnect),
//         }

//         let pkg_id = pkg.correlation;
//         let pkg_cmd = pkg.cmd;

//         let resp_opt = self.assocs.remove(&pkg_id).and_then(|req|
//         {
//             self.sessions.remove(&req.session).map(|session|
//             {
//                 Response {
//                     session,
//                     original_cmd: req.cmd,
//                 }
//             })
//         });

//         if let Some(mut resp) = resp_opt {
//             debug!("Package [{}]: command {:?} received {:?}.", pkg_id, resp.original_cmd, pkg_cmd);

//             let _ = match pkg.cmd {
//                 Cmd::BadRequest => {
//                     let msg = pkg.build_text();

//                     error!("Bad request for command {:?}: {}", resp.original_cmd, msg);

//                     resp.session.failed(OperationError::ServerError(Some(msg)));
//                 },

//                 Cmd::NotAuthenticated => {
//                     error!("Not authenticated for command {:?}.", resp.original_cmd);

//                     resp.session.failed(OperationError::AuthenticationRequired);
//                 },

//                 Cmd::NotHandled => {
//                     warn!("Not handled request {:?} id {}.", resp.original_cmd, pkg_id);

//                     let decoded_msg: io::Result<Option<types::Endpoint>> =
//                         pkg.to_message().and_then(|not_handled: messages::NotHandled|
//                         {
//                             if let messages::NotHandled_NotHandledReason::NotMaster = not_handled.get_reason() {
//                                 let master_info: messages::NotHandled_MasterInfo =
//                                     protobuf::parse_from_bytes(not_handled.get_additional_info())?;

//                                 // TODO - Support reconnection on the secure port when we are going to
//                                 // implement SSL connection.
//                                 let addr_str = format!("{}:{}", master_info.get_external_tcp_address(), master_info.get_external_tcp_port());
//                                 let external_tcp_port = types::Endpoint::from_addrs(addr_str)?;

//                                 Ok(Some(external_tcp_port))
//                             } else {
//                                 Ok(None)
//                             }
//                         });

//                     match decoded_msg {
//                         Ok(endpoint_opt) => {
//                             if let Some(endpoint) = endpoint_opt {
//                                 warn!("Received a non master error on command {:?} id {}, [{:?}]",
//                                       resp.original_cmd, pkg_id, endpoint);

//                                 awaiting.push(resp.session);
//                             } else {
//                                 warn!("The server has either not started or is too busy.
//                                       Retrying command {:?} id {}.", resp.original_cmd, pkg_id);

//                                 match resp.session.retry(&mut self.buffer, pkg_id) {
//                                     Ok(outcome) => {
//                                         let pkgs = outcome.produced_pkgs();

//                                         if !pkgs.is_empty() {
//                                             for pkg in pkgs.as_slice() {
//                                                 self.assocs.insert(pkg.correlation, Request::new(resp.session.id, pkg.cmd));
//                                             }

//                                             self.sessions.insert(resp.session.id, resp.session);
//                                             conn.enqueue_all(pkgs);
//                                         }
//                                     },

//                                     Err(error) => {
//                                         error!(
//                                             "An error occured when retrying command {:?} id {}: {}.",
//                                             resp.original_cmd, pkg_id, error
//                                         );
//                                     },
//                                 }

//                             }
//                         },

//                         Err(e) => {
//                             error!("Decoding error: can't decode NotHandled message: {}.", e);
//                         },
//                     };
//                 },

//                 _ => match resp.session.receive(conn.id, &mut self.buffer, pkg) {
//                     Ok(outcome) => {
//                         let pkgs = outcome.produced_pkgs();

//                         if !pkgs.is_empty() {
//                             for pkg in pkgs.as_slice() {
//                                 self.assocs.insert(pkg.correlation, Request::new(resp.session.id, pkg.cmd));
//                             }

//                             self.sessions.insert(resp.session.id, resp.session);
//                             conn.enqueue_all(pkgs);
//                         }
//                     },

//                     Err(e) => {
//                         error!("An error occured when running operation: {}", e);
//                         let msg = format!("Exception raised: {}", e);

//                         resp.session.failed(OperationError::InvalidOperation(msg));
//                     },
//                 },
//             };

//             None
//         } else {
//             warn!("Package [{}] not handled: cmd {:?}.", pkg_id, pkg_cmd);

//             None
//         }

//         // let pkg_id  = pkg.correlation;
//         // let pkg_cmd = pkg.cmd;

//         // if let Some(req) = self.assocs.remove(&pkg_id) {
//         //     let session_id = req.session;
//         //     let original_cmd = req.cmd;

//         //     debug!("Package [{}]: command {:?} received {:?}.", pkg_id, original_cmd, pkg_cmd);

//         //     let outcome = {
//         //         let mut op =
//         //             self.sessions
//         //                 .remove(&session_id)
//         //                 .expect(&format!("Unknown session!: Package[{}]: command {:?}, received {:?}.", pkg_id, original_cmd, pkg_cmd));

//         //         let out = {

//         //             match pkg.cmd {
//         //                 Cmd::BadRequest => {
//         //                     let msg = pkg.build_text();

//         //                     error!("Bad request for command {:?}: {}.", original_cmd, msg);

//         //                     op.failed(OperationError::ServerError(Some(msg)));

//         //                     Out::Failed
//         //                 },

//         //                 Cmd::NotAuthenticated => {
//         //                     error!("Not authenticated for command {:?}.", original_cmd);

//         //                     op.failed(OperationError::AuthenticationRequired);

//         //                     Out::Failed
//         //                 },

//         //                 Cmd::NotHandled => {
//         //                     warn!("Not handled request {:?} id {}.", original_cmd, pkg_id);

//         //                     let msg: io::Result<Option<types::Endpoint>> =
//         //                         pkg.to_message().and_then(|not_handled: messages::NotHandled|
//         //                         {
//         //                             if let messages::NotHandled_NotHandledReason::NotMaster = not_handled.get_reason() {
//         //                                 let master_info: messages::NotHandled_MasterInfo =
//         //                                     protobuf::parse_from_bytes(not_handled.get_additional_info())?;

//         //                                 // TODO - Support reconnection on the secure port when we are going to
//         //                                 // implement SSL connection.
//         //                                 let addr_str = format!("{}:{}", master_info.get_external_tcp_address(), master_info.get_external_tcp_port());
//         //                                 let external_tcp_port = types::Endpoint::from_addrs(addr_str)?;

//         //                                 Ok(Some(external_tcp_port))
//         //                             } else {
//         //                                 Ok(None)
//         //                             }
//         //                         });

//         //                     match msg {
//         //                         Ok(endpoint_opt) => {
//         //                             match endpoint_opt {
//         //                                 Some(endpoint) => {
//         //                                     warn!("Received a non master error on command {:?} id {}, [{:?}]", original_cmd, pkg_id, endpoint);

//         //                                     awaiting.push(op);

//         //                                     Out::Handled(Some(endpoint))
//         //                                 },

//         //                                 None => {
//         //                                     warn!("The server has either not started or is too busy.
//         //                                           Retrying command {:?} id {}.", original_cmd, pkg_id);

//         //                                     match op.retry(&mut self.buffer, pkg_id) {
//         //                                         Ok(outcome) => {
//         //                                             let pkgs = outcome.produced_pkgs();

//         //                                             if !pkgs.is_empty() {
//         //                                                 for pkg in pkgs.as_slice() {
//         //                                                     self.assocs.insert(pkg.correlation, Request::new(session_id, pkg.cmd));
//         //                                                 }

//         //                                                 self.sessions.insert(session_id, op);
//         //                                                 conn.enqueue_all(pkgs);
//         //                                             }

//         //                                             Out::Handled(None)
//         //                                         },

//         //                                         Err(error) => {
//         //                                             error!(
//         //                                                 "An error occured when retrying command {:?} id {}: {}.",
//         //                                                 original_cmd, pkg_id, error
//         //                                             );

//         //                                             Out::Failed
//         //                                         },
//         //                                     }
//         //                                 },
//         //                             }
//         //                         },

//         //                         Err(error) => {
//         //                             error!("Decoding error: can't decode NotHandled message: {}.", error);

//         //                             Out::Failed
//         //                         },
//         //                     }
//         //                 },

//         //                 _ => match op.receive(conn.id, &mut self.buffer, pkg) {
//         //                     Ok(outcome) => {
//         //                         let is_continue = outcome.is_continue();
//         //                         let pkgs = outcome.produced_pkgs();

//         //                         if !pkgs.is_empty() {
//         //                             for pkg in pkgs.as_slice() {
//         //                                 self.assocs.insert(pkg.correlation, Request::new(session_id, pkg.cmd));
//         //                             }

//         //                             self.sessions.insert(session_id, op);
//         //                             conn.enqueue_all(pkgs);
//         //                         }

//         //                         if is_continue {
//         //                             self.assocs.insert(pkg_id, req);
//         //                         }

//         //                         Out::Handled(None)
//         //                     },

//         //                     Err(e) => {
//         //                         error!("An error occured when running operation: {}", e);
//         //                         let msg = format!("Exception raised: {}", e);

//         //                         op.failed(OperationError::InvalidOperation(msg));

//         //                         Out::Failed
//         //                     },
//         //                 },
//         //             }
//         //         };

//         //         match out {
//         //             Out::Failed => {
//         //                 for req_id in op.request_keys() {
//         //                     self.assocs.remove(req_id);
//         //                 }

//         //                 None
//         //             },

//         //             Out::Handled(force_reconnect) => {
//         //                 force_reconnect
//         //             },
//         //         }
//         //     };

//         //     outcome
//         // } else {
//         //     warn!("Package [{}] not handled: cmd {:?}.", pkg_id, pkg_cmd);

//         //     None
//         // }
//     }

//     fn check_and_retry(&mut self, conn: &Connection) {
//         let mut sessions_to_delete = Vec::new();

//         for op in self.sessions.values_mut() {

//             let result = op.check_and_retry(conn.id, &mut self.buffer);

//             match result {
//                 Ok(outcome) => {
//                     if outcome.is_done() {
//                         for id in op.request_keys() {
//                             self.assocs.remove(id);
//                         }

//                         sessions_to_delete.push(op.id);
//                     } else {
//                         let pkgs = outcome.produced_pkgs();

//                         if !pkgs.is_empty() {
//                             for pkg in pkgs.as_slice() {
//                                 self.assocs.insert(pkg.correlation, Request::new(op.id, pkg.cmd));
//                             }

//                             conn.enqueue_all(pkgs);
//                         }
//                     }
//                 },

//                 Err(e) => {
//                     error!("Exception raised when checking out operation: {}", e);
//                     let msg = format!("Exception raised: {}", e);

//                     op.failed(OperationError::InvalidOperation(msg));

//                     for id in op.request_keys() {
//                         self.assocs.remove(id);
//                     }

//                     sessions_to_delete.push(op.id);
//                 },
//             }
//         }

//         for session_id in sessions_to_delete {
//             self.sessions.remove(&session_id);
//         }
//     }

//     pub(crate) fn abort(&mut self) {
//         for op in self.sessions.values_mut() {
//             op.failed(OperationError::Aborted);
//         }
//     }
// }

pub(crate) struct Registry {
    pendings: HashMap<Uuid, Tracker>,
    awaiting: Vec<OperationWrapper>,
    buffer: BytesMut,
}

impl Registry {
    pub(crate) fn new() -> Registry {
        Registry {
            pendings: HashMap::new(),
            awaiting: Vec::new(),
            buffer: BytesMut::new(),
        }
    }

    pub(crate) fn register(&mut self, op: OperationWrapper, conn: Option<&Connection>) {
        match conn {
            None       => self.awaiting.push(op),
            Some(conn) => register_operation(self, conn, op),
        }
    }

    pub(crate) fn handle(&mut self, pkg: Pkg, conn: &Connection) -> Option<types::Endpoint> {
        handle_pkg(self, conn, pkg)
    }

    pub(crate) fn check_and_retry(&mut self, conn: &Connection) {
        check_and_retry(self, conn);

        while let Some(op) = self.awaiting.pop() {
            self.register(op, Some(conn));
        }
    }

    pub(crate) fn abort(&mut self) {
        abort(self);

        for op in self.awaiting.iter_mut() {
            op.failed(OperationError::Aborted);
        }
    }
}

pub(crate) struct Tracker {
    cmd: Cmd,
    attempts: usize,
    started: Instant,
    lasting: bool,
    conn_id: Uuid,
    operation: OperationWrapper,
}

impl Tracker {
    fn new(cmd: Cmd, conn_id: Uuid, operation: OperationWrapper) -> Tracker {
        Tracker {
            cmd,
            attempts: 0,
            started: Instant::now(),
            lasting: false,
            conn_id,
            operation,
        }
    }

    fn reuse(&mut self, conn: &Connection) {
        self.attempts = 0;
        self.started = Instant::now();
        self.conn_id = conn.id;
        self.lasting = false;
    }

    fn has_timeout(&self, timeout: Duration) -> bool {
        !self.lasting && self.started.elapsed() >= timeout
    }
}

fn register_operation(registry: &mut Registry, conn: &Connection, operation: OperationWrapper)
{
    let correlation = Uuid::new_v4();

    match operation.issue_request(&mut registry.buffer, correlation) {
        Ok(pkg) => {
            debug!("Issuing request {} for {:?}", pkg.correlation, pkg.cmd);

            let tracker = Tracker::new(pkg.cmd, conn.id, operation);

            registry.pendings.insert(correlation, tracker);
            conn.enqueue(pkg);
        },

        Err(e) => {
            error!("Exception occured when issuing a request: {}", e);
        }
    }
}

fn retry_operation(registry: &mut Registry, conn: &Connection, mut tracker: Tracker, correlation: Uuid)
    -> io::Result<()>
{
    if tracker.attempts + 1 >= tracker.operation.max_retry {
        tracker.operation.failed(OperationError::Aborted);

        return Ok(());
    }

    let pkg = tracker.operation.retry(&mut registry.buffer, tracker.cmd, correlation)?;

    tracker.attempts += 1;
    tracker.conn_id = conn.id;

    conn.enqueue(pkg);
    registry.pendings.insert(correlation, tracker);

    Ok(())
}

fn operation_receive(registry: &mut Registry, conn: &Connection, mut tracker: Tracker, pkg: Pkg)
    -> io::Result<()>
{
    let pkg_id = pkg.correlation;
    let pkg_cmd = pkg.cmd;
    let (result, pkg_opt) = tracker.operation.respond(&mut registry.buffer, pkg)?;

    match result {
        ImplResult::Retry => {
            return retry_operation(registry, conn, tracker, pkg_id);
        },

        ImplResult::Done => {
            if let Some(pkg) = pkg_opt {
                tracker.reuse(conn);
                conn.enqueue(pkg);
                registry.pendings.insert(pkg_id, tracker);
            }
        },

        ImplResult::Awaiting => {
            tracker.lasting = true;

            registry.pendings.insert(pkg_id, tracker);
        },

        // This enum value is probably no longer useful.
        ImplResult::Terminate => {},

        ImplResult::Unexpected => {
            tracker.operation.failed(OperationError::wrong_client_impl_on_cmd(pkg_cmd));
        },
    };

    Ok(())
}

fn handle_pkg(registry: &mut Registry, conn: &Connection, pkg: Pkg) -> Option<types::Endpoint>
{
    if let Some(mut tracker) = registry.pendings.remove(&pkg.correlation) {
        debug!("Package [{}]: command {:?} received {:?}.", pkg.correlation, tracker.cmd, pkg.cmd);

        match pkg.cmd {
            Cmd::BadRequest => {
                let msg = pkg.build_text();

                error!("Bad request for command {:?}: {}", tracker.cmd, msg);

                tracker.operation.failed(OperationError::ServerError(Some(msg)));

                None
            },

            Cmd::NotAuthenticated => {
                error!("Not authenticated for command {:?}.", tracker.cmd);

                tracker.operation.failed(OperationError::AuthenticationRequired);

                None
            },

            Cmd::NotHandled => {
                warn!("Not handled request {:?} id {}.", tracker.cmd, pkg.correlation);

                let decoded_msg: io::Result<Option<types::Endpoint>> =
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

                match decoded_msg {
                    Ok(endpoint_opt) => {
                        if let Some(endpoint) = endpoint_opt {
                            warn!("Received a non master error on command {:?} id {}, [{:?}]",
                                  tracker.cmd, pkg.correlation, endpoint);

                            registry.awaiting.push(tracker.operation);

                            return Some(endpoint);
                        }

                        warn!("The server has either not started or is too busy.
                              Retrying command {:?} id {}.", tracker.cmd, pkg.correlation);

                        let original_cmd = tracker.cmd;

                        if let Err(error) = retry_operation(registry, conn, tracker, pkg.correlation) {
                            error!(
                                "An error occured when retrying command {:?} id {}: {}.",
                                original_cmd, pkg.correlation, error
                            );
                        }

                        None
                    },

                    Err(e) => {
                        error!("Decoding error: can't decode NotHandled message: {}.", e);

                        None
                    },
                }
            },


            _ => {
                let pkg_id = pkg.correlation;
                let original_cmd = tracker.cmd;
                let resp_cmd = pkg.cmd;

                if let Err(error) = operation_receive(registry, conn, tracker, pkg) {
                    error!(
                        "An error occured when running operation {}. command: {:?}, response: {:?}: {}",
                        pkg_id, original_cmd, resp_cmd, error
                    );
                }

                None
            },
        }
    } else {
        None
    }
}

fn check_and_retry(registry: &mut Registry, conn: &Connection) {
    use std::mem;

    let pendings = mem::replace(&mut registry.pendings, HashMap::new());

    for (key, mut tracker) in pendings {
        if tracker.conn_id != conn.id {
            match tracker.operation.connection_has_dropped(&mut registry.buffer, tracker.cmd) {
                Ok(pkg_opt) => {
                    if let Some(pkg) = pkg_opt {
                        tracker.cmd = pkg.cmd;

                        tracker.reuse(conn);
                        registry.pendings.insert(pkg.correlation, tracker);
                        conn.enqueue(pkg);
                    }
                },

                Err(error) => {
                    error!("Error on connection_has_dropped function: {}", error);

                    tracker.operation.failed(OperationError::Aborted);
                },
            }
        }
    }
}

fn abort(registry: &mut Registry) {

}
