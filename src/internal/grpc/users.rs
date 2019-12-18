#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<create_req::Options>,
}
pub mod create_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
        #[prost(string, tag = "2")]
        pub password: std::string::String,
        #[prost(string, tag = "3")]
        pub full_name: std::string::String,
        #[prost(string, repeated, tag = "4")]
        pub groups: ::std::vec::Vec<std::string::String>,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateResp {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<update_req::Options>,
}
pub mod update_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
        #[prost(string, tag = "2")]
        pub password: std::string::String,
        #[prost(string, tag = "3")]
        pub full_name: std::string::String,
        #[prost(string, repeated, tag = "4")]
        pub groups: ::std::vec::Vec<std::string::String>,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateResp {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<delete_req::Options>,
}
pub mod delete_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteResp {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnableReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<enable_req::Options>,
}
pub mod enable_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EnableResp {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisableReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<disable_req::Options>,
}
pub mod disable_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisableResp {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetailsReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<details_req::Options>,
}
pub mod details_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DetailsResp {
    #[prost(message, optional, tag = "1")]
    pub user_details: ::std::option::Option<details_resp::UserDetails>,
}
pub mod details_resp {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct UserDetails {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
        #[prost(string, tag = "2")]
        pub full_name: std::string::String,
        #[prost(string, repeated, tag = "3")]
        pub groups: ::std::vec::Vec<std::string::String>,
        #[prost(string, tag = "4")]
        pub last_updated: std::string::String,
        #[prost(bool, tag = "5")]
        pub disabled: bool,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangePasswordReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<change_password_req::Options>,
}
pub mod change_password_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
        #[prost(string, tag = "2")]
        pub current_password: std::string::String,
        #[prost(string, tag = "3")]
        pub new_password: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChangePasswordResp {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResetPasswordReq {
    #[prost(message, optional, tag = "1")]
    pub options: ::std::option::Option<reset_password_req::Options>,
}
pub mod reset_password_req {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Options {
        #[prost(string, tag = "1")]
        pub login_name: std::string::String,
        #[prost(string, tag = "2")]
        pub new_password: std::string::String,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResetPasswordResp {}
#[doc = r" Generated server implementations."]
pub mod users_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct UsersClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl UsersClient<tonic::transport::Channel> {
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
    impl<T> UsersClient<T>
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
        pub async fn create(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateReq>,
        ) -> Result<tonic::Response<super::CreateResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/Create");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateReq>,
        ) -> Result<tonic::Response<super::UpdateResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/Update");
            self.inner.unary(request.into_request(), path, codec).await
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
            let path = http::uri::PathAndQuery::from_static("/users.Users/Delete");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn disable(
            &mut self,
            request: impl tonic::IntoRequest<super::DisableReq>,
        ) -> Result<tonic::Response<super::DisableResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/Disable");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn enable(
            &mut self,
            request: impl tonic::IntoRequest<super::EnableReq>,
        ) -> Result<tonic::Response<super::EnableResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/Enable");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn details(
            &mut self,
            request: impl tonic::IntoRequest<super::DetailsReq>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::DetailsResp>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/Details");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn change_password(
            &mut self,
            request: impl tonic::IntoRequest<super::ChangePasswordReq>,
        ) -> Result<tonic::Response<super::ChangePasswordResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/ChangePassword");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn reset_password(
            &mut self,
            request: impl tonic::IntoRequest<super::ResetPasswordReq>,
        ) -> Result<tonic::Response<super::ResetPasswordResp>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/users.Users/ResetPassword");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for UsersClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
}
