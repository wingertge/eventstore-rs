use byteorder::{ ByteOrder, BigEndian };
use crate::es6::grpc::streams;
use crate::es6::types::{ ResolvedEvent, RecordedEvent, Position };
use futures_3::stream::{ Stream, StreamExt };
use uuid::Uuid;

fn read_resp_to_resolved_event(
    resp: streams::ReadResp,
) -> ResolvedEvent {
    let mut resp = resp
        .event
        .expect("Will see later if it's a possible situation");

    ResolvedEvent {
        event: resp.event.take().map(server_event_to_client_event),
        link: resp.link.take().map(server_event_to_client_event),
        position: None,
    }
}

fn server_event_to_client_event(
    mut src: streams::read_resp::read_event::RecordedEvent,
) -> RecordedEvent {
    let event_id = src
        .id
        .take()
        .map(raw_uuid_to_uuid)
        .expect("UUID property en RecordedEvent must be present");

    let position = Position {
        commit: src.commit_position,
        prepare: src.prepare_position,
    };

    RecordedEvent {
        event_stream_id: src.stream_name,
        stream_revision:  src.stream_revision,
        data: src.data,
        metadata: src.metadata,
        event_type: "<not known yet>".to_owned(),
        position,
        event_id,
    }
}

fn raw_uuid_to_uuid(
    src: streams::Uuid,
) -> Uuid {
    let value = src.value.expect("I expect Uuid value to be defined for now");

    match value {
        streams::uuid::Value::Structured(s) => {
            let mut buf = vec![];

            BigEndian::write_i64(&mut buf, s.most_significant_bits);
            BigEndian::write_i64(&mut buf, s.least_significant_bits);

            uuid::Uuid::from_slice(buf.as_slice())
                .expect("We should have a valid UUID out of this")
        },

        streams::uuid::Value::String(s) => {
            s.parse().expect("I expect to have a valid uuid from this message.")
        },
    }
}

/// A command that reads several events from a stream. It can read events
/// forward or backward.
pub struct ReadStreamEvents {
    client: streams::streams_client::StreamsClient<tonic::transport::Channel>,
    stream: String,
    max_count: i32,
    start: i64,
    resolve_link_tos: bool,
    read_direction: streams::read_req::options::ReadDirection,
}

impl ReadStreamEvents {
    pub(crate) fn new(
        client: streams::streams_client::StreamsClient<tonic::transport::Channel>,
        stream: String,
    ) -> Self {
        ReadStreamEvents {
            client,
            stream,
            max_count: 500,
            start: 0,
            resolve_link_tos: false,
            read_direction: streams::read_req::options::ReadDirection::Forwards,
        }
    }

    /// Asks the command to read forward (toward the end of the stream).
    /// That's the default behavior.
    pub fn forward(self) -> Self {
        ReadStreamEvents {
            read_direction: streams::read_req::options::ReadDirection::Forwards,
            ..self
        }
    }

    /// Asks the command to read backward (toward the begining of the stream).
    pub fn backward(self) -> Self {
        ReadStreamEvents {
            read_direction: streams::read_req::options::ReadDirection::Backwards,
            ..self
        }
    }

    /// Max batch size.
    pub fn max_count(self, max_count: i32) -> Self {
        ReadStreamEvents {
            max_count,
            ..self
        }
    }

    /// Starts the read at the given event number. By default, it starts at
    /// 0.
    pub fn start_from(self, start: i64) -> Self {
        ReadStreamEvents {
            start,
            ..self
        }
    }

    /// Starts the read from the beginning of the stream. It also set the read
    /// direction to `Forward`.
    pub fn start_from_beginning(self) -> Self {
        ReadStreamEvents {
            start: 0,
            read_direction: streams::read_req::options::ReadDirection::Forwards,
            ..self
        }
    }

    /// Starts the read from the end of the stream. It also set the read
    /// direction to `Backward`.
    pub fn start_from_end_of_stream(self) -> Self {
        ReadStreamEvents {
            start: -1,
            read_direction: streams::read_req::options::ReadDirection::Backwards,
            ..self
        }
    }
/// When using projections, you can have links placed into another stream.
    /// By resolving links, the server will resolve those links and will return
    /// the event that the link points to.
    pub fn resolve_links(self) -> Self {
        ReadStreamEvents {
            resolve_link_tos: true,
            ..self
        }
    }

    /// When using projections, you can have links placed into another stream.
    /// By resolving links, the server will resolve those links and will return
    /// the event that the link points to.
    pub fn no_resolve_links(self) -> Self {
        ReadStreamEvents {
            resolve_link_tos: false,
            ..self
        }
    }

    pub async fn execute(
        mut self
    ) -> Result<impl Stream<Item=Result<ResolvedEvent, tonic::Status>>, tonic::Status> {
        use streams::read_req::options::{
            CountOption,
            StreamOption,
            StreamOptions,
        };

        let stream_options = StreamOptions {
            stream_name: self.stream,
            revision_options: None,
        };

        let options = streams::read_req::Options {
            read_direction: self.read_direction as i32,
            resolve_links: self.resolve_link_tos,
            stream_option: Some(StreamOption::Stream(stream_options)),
            count_option: Some(CountOption::Count(self.max_count)),
            filter_option: None,
        };

        let req = streams::ReadReq {
            options: Some(options),
        };

        let streaming: tonic::Streaming<streams::ReadResp> =
            self.client.read(tonic::Request::new(req))
                .await?
                .into_inner();

        let stream = streaming.map(|res| {
            res.map(read_resp_to_resolved_event)
        });

        Ok(stream)
    }

    // /// Sends asynchronously the read command to the server.
    // pub fn execute(self) -> impl Future<Item=types::ReadStreamStatus<types::StreamSlice>, Error=OperationError> {
    //     let     (rcv, promise) = operations::Promise::new(1);
    //     let mut op             = operations::ReadStreamEvents::new(promise, self.direction);

    //     op.set_event_stream_id(self.stream);
    //     op.set_from_event_number(self.start);
    //     op.set_max_count(self.max_count);
    //     op.set_require_master(self.require_master);
    //     op.set_resolve_link_tos(self.resolve_link_tos);

    //     let op = operations::OperationWrapper::new(op,
    //                                                self.creds,
    //                                                self.settings.operation_retry.to_usize(),
    //                                                self.settings.operation_timeout);

    //     self.sender.send(Msg::new_op(op)).wait().unwrap();

    //     single_value_future(rcv)
    // }
}
