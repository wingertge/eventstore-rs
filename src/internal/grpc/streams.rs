#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<read_req::Options>,
}
pub mod read_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(enumeration = "options::ReadDirection", tag = "3")]
        pub read_direction: i32,
        #[prost(bool, tag = "4")]
        pub resolve_links: bool,
        #[prost(oneof = "options::StreamOptions", tags = "1, 2")]
        pub stream_options: ::std::option::Option<options::StreamOptions>,
        #[prost(oneof = "options::CountOptions", tags = "5, 6")]
        pub count_options: ::std::option::Option<options::CountOptions>,
        #[prost(oneof = "options::FilterOptions", tags = "7, 8")]
        pub filter_options: ::std::option::Option<options::FilterOptions>,
    }
    pub mod options {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct StreamOptions {
            #[prost(string, tag = "1")]
            pub stream_name: std::string::String,
            #[prost(oneof = "stream_options::RevisionOptions", tags = "2, 3, 4")]
            pub revision_options: ::std::option::Option<stream_options::RevisionOptions>,
        }
        pub mod stream_options {
            #[derive(Clone, PartialEq, ::prost::Oneof)]
            pub enum RevisionOptions {
                #[prost(uint64, tag = "2")]
                Revision(u64),
                #[prost(message, tag = "3")]
                Start(super::super::Empty),
                #[prost(message, tag = "4")]
                End(super::super::Empty),
            }
        }
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct AllOptions {
            #[prost(oneof = "all_options::AllOptions", tags = "1, 2, 3")]
            pub all_options: ::std::option::Option<all_options::AllOptions>,
        }
        pub mod all_options {
            #[derive(Clone, PartialEq, ::prost::Oneof)]
            pub enum AllOptions {
                #[prost(message, tag = "1")]
                Position(super::Position),
                #[prost(message, tag = "2")]
                Start(super::super::Empty),
                #[prost(message, tag = "3")]
                End(super::super::Empty),
            }
        }
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct SubscriptionOptions {}
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Position {
            #[prost(uint64, tag = "1")]
            pub commit_position: u64,
            #[prost(uint64, tag = "2")]
            pub prepare_position: u64,
        }
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct FilterOptions {
            #[prost(oneof = "filter_options::Filter", tags = "1, 2")]
            pub filter: ::std::option::Option<filter_options::Filter>,
            #[prost(oneof = "filter_options::Window", tags = "3, 4")]
            pub window: ::std::option::Option<filter_options::Window>,
        }
        pub mod filter_options {
            #[derive(Clone, PartialEq, ::prost::Message)]
            pub struct Expression {
                #[prost(string, tag = "1")]
                pub regex: std::string::String,
                #[prost(string, repeated, tag = "2")]
                pub prefix: ::std::vec::Vec<std::string::String>,
            }
            #[derive(Clone, PartialEq, ::prost::Oneof)]
            pub enum Filter {
                #[prost(message, tag = "1")]
                StreamName(Expression),
                #[prost(message, tag = "2")]
                EventType(Expression),
            }
            #[derive(Clone, PartialEq, ::prost::Oneof)]
            pub enum Window {
                #[prost(int32, tag = "3")]
                Max(i32),
                #[prost(message, tag = "4")]
                Count(super::super::Empty),
            }
        }
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
        #[repr(i32)]
        pub enum ReadDirection {
            Forwards = 0,
            Backwards = 1,
        }
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum StreamOptions {
            #[prost(message, tag = "1")]
            Stream(StreamOptions),
            #[prost(message, tag = "2")]
            All(AllOptions),
        }
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum CountOptions {
            #[prost(int32, tag = "5")]
            Count(i32),
            #[prost(message, tag = "6")]
            Subscription(SubscriptionOptions),
        }
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum FilterOptions {
            #[prost(message, tag = "7")]
            Filter(FilterOptions),
            #[prost(message, tag = "8")]
            NoFilter(super::Empty),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadResp {
    #[prost(message, optional, tag = "1")]
    pub event: ::std::option::Option<read_resp::ReadEvent>,
}
pub mod read_resp {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ReadEvent {
        #[prost(message, optional, tag = "1")]
        pub event: ::std::option::Option<read_event::RecordedEvent>,
        #[prost(message, optional, tag = "2")]
        pub link: ::std::option::Option<read_event::RecordedEvent>,
        #[prost(oneof = "read_event::Position", tags = "3, 4")]
        pub position: ::std::option::Option<read_event::Position>,
    }
    pub mod read_event {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct RecordedEvent {
            #[prost(message, optional, tag = "1")]
            pub id: ::std::option::Option<super::super::Uuid>,
            #[prost(string, tag = "2")]
            pub stream_name: std::string::String,
            #[prost(uint64, tag = "3")]
            pub stream_revision: u64,
            #[prost(uint64, tag = "4")]
            pub prepare_position: u64,
            #[prost(uint64, tag = "5")]
            pub commit_position: u64,
            #[prost(map = "string, string", tag = "6")]
            pub metadata: ::std::collections::HashMap<std::string::String, std::string::String>,
            #[prost(bytes, tag = "7")]
            pub custom_metadata: std::vec::Vec<u8>,
            #[prost(bytes, tag = "8")]
            pub data: std::vec::Vec<u8>,
        }
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Position {
            #[prost(uint64, tag = "3")]
            CommitPosition(u64),
            #[prost(message, tag = "4")]
            NoPosition(super::Empty),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendReq {
    #[prost(oneof = "append_req::Content", tags = "1, 2")]
    pub content: ::std::option::Option<append_req::Content>,
}
pub mod append_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub stream_name: std::string::String,
        #[prost(oneof = "options::ExpectedStreamRevision", tags = "2, 3, 4, 5")]
        pub expected_stream_revision: ::std::option::Option<options::ExpectedStreamRevision>,
    }
    pub mod options {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ExpectedStreamRevision {
            #[prost(uint64, tag = "2")]
            Revision(u64),
            #[prost(message, tag = "3")]
            NoStream(super::Empty),
            #[prost(message, tag = "4")]
            Any(super::Empty),
            #[prost(message, tag = "5")]
            StreamExists(super::Empty),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ProposedMessage {
        #[prost(message, optional, tag = "1")]
        pub id: ::std::option::Option<super::Uuid>,
        #[prost(map = "string, string", tag = "2")]
        pub metadata: ::std::collections::HashMap<std::string::String, std::string::String>,
        #[prost(bytes, tag = "3")]
        pub custom_metadata: std::vec::Vec<u8>,
        #[prost(bytes, tag = "4")]
        pub data: std::vec::Vec<u8>,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Content {
        #[prost(message, tag = "1")]
        Options(Options),
        #[prost(message, tag = "2")]
        ProposedMessage(ProposedMessage),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppendResp {
    #[prost(oneof = "append_resp::CurrentRevisionOptions", tags = "1, 2")]
    pub current_revision_options: ::std::option::Option<append_resp::CurrentRevisionOptions>,
    #[prost(oneof = "append_resp::PositionOptions", tags = "3, 4")]
    pub position_options: ::std::option::Option<append_resp::PositionOptions>,
}
pub mod append_resp {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Position {
        #[prost(uint64, tag = "1")]
        pub commit_position: u64,
        #[prost(uint64, tag = "2")]
        pub prepare_position: u64,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum CurrentRevisionOptions {
        #[prost(uint64, tag = "1")]
        CurrentRevision(u64),
        #[prost(message, tag = "2")]
        NoStream(Empty),
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum PositionOptions {
        #[prost(message, tag = "3")]
        Position(Position),
        #[prost(message, tag = "4")]
        Empty(Empty),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<delete_req::Options>,
}
pub mod delete_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub stream_name: std::string::String,
        #[prost(oneof = "options::ExpectedStreamRevision", tags = "2, 3, 4, 5")]
        pub expected_stream_revision: ::std::option::Option<options::ExpectedStreamRevision>,
    }
    pub mod options {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ExpectedStreamRevision {
            #[prost(uint64, tag = "2")]
            Revision(u64),
            #[prost(message, tag = "3")]
            NoStream(super::Empty),
            #[prost(message, tag = "4")]
            Any(super::Empty),
            #[prost(message, tag = "5")]
            StreamExists(super::Empty),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResp {
    #[prost(oneof = "delete_resp::PositionOptions", tags = "1, 2")]
    pub position_options: ::std::option::Option<delete_resp::PositionOptions>,
}
pub mod delete_resp {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Position {
        #[prost(uint64, tag = "1")]
        pub commit_position: u64,
        #[prost(uint64, tag = "2")]
        pub prepare_position: u64,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum PositionOptions {
        #[prost(message, tag = "1")]
        Position(Position),
        #[prost(message, tag = "2")]
        Empty(Empty),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TombstoneReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<tombstone_req::Options>,
}
pub mod tombstone_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub stream_name: std::string::String,
        #[prost(oneof = "options::ExpectedStreamRevision", tags = "2, 3, 4, 5")]
        pub expected_stream_revision: ::std::option::Option<options::ExpectedStreamRevision>,
    }
    pub mod options {
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum ExpectedStreamRevision {
            #[prost(uint64, tag = "2")]
            Revision(u64),
            #[prost(message, tag = "3")]
            NoStream(super::Empty),
            #[prost(message, tag = "4")]
            Any(super::Empty),
            #[prost(message, tag = "5")]
            StreamExists(super::Empty),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TombstoneResp {
    #[prost(oneof = "tombstone_resp::PositionOptions", tags = "1, 2")]
    pub position_options: ::std::option::Option<tombstone_resp::PositionOptions>,
}
pub mod tombstone_resp {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Position {
        #[prost(uint64, tag = "1")]
        pub commit_position: u64,
        #[prost(uint64, tag = "2")]
        pub prepare_position: u64,
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Empty {}
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum PositionOptions {
        #[prost(message, tag = "1")]
        Position(Position),
        #[prost(message, tag = "2")]
        Empty(Empty),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Uuid {
    #[prost(oneof = "uuid::Value", tags = "1, 2")]
    pub value: ::std::option::Option<uuid::Value>,
}
pub mod uuid {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Structured {
        #[prost(int64, tag = "1")]
        pub most_significant_bits: i64,
        #[prost(int64, tag = "2")]
        pub least_significant_bits: i64,
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Structured(Structured),
        #[prost(string, tag = "2")]
        String(std::string::String),
    }
}
#[doc = r" Generated server implementations."]
pub mod streams_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct StreamsClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl StreamsClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> StreamsClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub async fn read(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadReq>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::ReadResp>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streams.Streams/Read");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn append(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::AppendReq>,
        ) -> Result<tonic::Response<super::AppendResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streams.Streams/Append");
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
        pub async fn delete(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteReq>,
        ) -> Result<tonic::Response<super::DeleteResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streams.Streams/Delete");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn tombstone(
            &mut self,
            request: impl tonic::IntoRequest<super::TombstoneReq>,
        ) -> Result<tonic::Response<super::TombstoneResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/streams.Streams/Tombstone");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for StreamsClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
