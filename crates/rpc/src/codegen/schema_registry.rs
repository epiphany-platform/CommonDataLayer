#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewSchema {
    /// for replication (empty for master, UUID for slaves)
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub query_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub insert_destination: ::prost::alloc::string::String,
    #[prost(string, required, tag = "5")]
    pub definition: ::prost::alloc::string::String,
    #[prost(enumeration = "schema_type::Type", required, tag = "6")]
    pub schema_type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schema {
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub insert_destination: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub query_address: ::prost::alloc::string::String,
    #[prost(enumeration = "schema_type::Type", required, tag = "4")]
    pub schema_type: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewSchemaVersion {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub definition: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaMetadataUpdate {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub insert_destination: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(enumeration = "schema_type::Type", optional, tag = "5")]
    pub schema_type: ::core::option::Option<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewSchemaView {
    #[prost(string, required, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
    /// for replication (empty for master, UUID for slaves)
    #[prost(string, required, tag = "2")]
    pub view_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_addr: ::prost::alloc::string::String,
    #[prost(string, required, tag = "5")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(string, required, tag = "6")]
    pub fields: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdatedView {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub materializer_addr: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub materializer_options: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub fields: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct View {
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub materializer_addr: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub fields: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionedId {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub version_req: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaDefinition {
    #[prost(string, required, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub definition: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaInsertDestination {
    #[prost(string, required, tag = "1")]
    pub insert_destination: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaQueryAddress {
    #[prost(string, required, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Id {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaVersions {
    #[prost(string, repeated, tag = "1")]
    pub versions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schemas {
    #[prost(map = "string, message", tag = "1")]
    pub schemas: ::std::collections::HashMap<::prost::alloc::string::String, Schema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaNames {
    #[prost(map = "string, string", tag = "1")]
    pub names:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaViews {
    #[prost(map = "string, message", tag = "1")]
    pub views: ::std::collections::HashMap<::prost::alloc::string::String, View>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewSchema {
    #[prost(string, required, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
    #[prost(message, required, tag = "2")]
    pub schema: Schema,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueToValidate {
    #[prost(string, required, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Errors {
    #[prost(string, repeated, tag = "1")]
    pub errors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PodName {
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaType {
    #[prost(enumeration = "schema_type::Type", required, tag = "1")]
    pub schema_type: i32,
}
/// Nested message and enum types in `SchemaType`.
pub mod schema_type {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        DocumentStorage = 0,
        Timeseries = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
#[doc = r" Generated client implementations."]
pub mod schema_registry_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct SchemaRegistryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SchemaRegistryClient<tonic::transport::Channel> {
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
    impl<T> SchemaRegistryClient<T>
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
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn add_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::NewSchema>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/AddSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_schema_version(
            &mut self,
            request: impl tonic::IntoRequest<super::NewSchemaVersion>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/AddSchemaVersion",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_schema_metadata(
            &mut self,
            request: impl tonic::IntoRequest<super::SchemaMetadataUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/UpdateSchemaMetadata",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_view_to_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::NewSchemaView>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/AddViewToSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn update_view(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdatedView>,
        ) -> Result<tonic::Response<super::View>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/UpdateView");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::VersionedId>,
        ) -> Result<tonic::Response<super::SchemaDefinition>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/GetSchema");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_metadata(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetSchemaMetadata",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_insert_destination(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaInsertDestination>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetSchemaInsertDestination",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_query_address(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaQueryAddress>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetSchemaQueryAddress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_versions(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaVersions>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetSchemaVersions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_type(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaType>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetSchemaType",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_view(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::View>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/GetView");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_schemas(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::Schemas>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllSchemas",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_schema_names(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::SchemaNames>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllSchemaNames",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_all_views_of_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllViewsOfSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_base_schema_of_view(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::ViewSchema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetBaseSchemaOfView",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn validate_value(
            &mut self,
            request: impl tonic::IntoRequest<super::ValueToValidate>,
        ) -> Result<tonic::Response<super::Errors>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/ValidateValue",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn promote_to_master(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::PodName>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/PromoteToMaster",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/Heartbeat");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for SchemaRegistryClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for SchemaRegistryClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "SchemaRegistryClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod schema_registry_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with SchemaRegistryServer."]
    #[async_trait]
    pub trait SchemaRegistry: Send + Sync + 'static {
        async fn add_schema(
            &self,
            request: tonic::Request<super::NewSchema>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status>;
        async fn add_schema_version(
            &self,
            request: tonic::Request<super::NewSchemaVersion>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn update_schema_metadata(
            &self,
            request: tonic::Request<super::SchemaMetadataUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn add_view_to_schema(
            &self,
            request: tonic::Request<super::NewSchemaView>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status>;
        async fn update_view(
            &self,
            request: tonic::Request<super::UpdatedView>,
        ) -> Result<tonic::Response<super::View>, tonic::Status>;
        async fn get_schema(
            &self,
            request: tonic::Request<super::VersionedId>,
        ) -> Result<tonic::Response<super::SchemaDefinition>, tonic::Status>;
        async fn get_schema_metadata(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status>;
        async fn get_schema_insert_destination(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaInsertDestination>, tonic::Status>;
        async fn get_schema_query_address(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaQueryAddress>, tonic::Status>;
        async fn get_schema_versions(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaVersions>, tonic::Status>;
        async fn get_schema_type(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaType>, tonic::Status>;
        async fn get_view(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::View>, tonic::Status>;
        async fn get_all_schemas(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Schemas>, tonic::Status>;
        async fn get_all_schema_names(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::SchemaNames>, tonic::Status>;
        async fn get_all_views_of_schema(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status>;
        async fn get_base_schema_of_view(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::ViewSchema>, tonic::Status>;
        async fn validate_value(
            &self,
            request: tonic::Request<super::ValueToValidate>,
        ) -> Result<tonic::Response<super::Errors>, tonic::Status>;
        async fn promote_to_master(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::PodName>, tonic::Status>;
        async fn heartbeat(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SchemaRegistryServer<T: SchemaRegistry> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: SchemaRegistry> SchemaRegistryServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T, B> Service<http::Request<B>> for SchemaRegistryServer<T>
    where
        T: SchemaRegistry,
        B: HttpBody + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/schema_registry.SchemaRegistry/AddSchema" => {
                    #[allow(non_camel_case_types)]
                    struct AddSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::NewSchema> for AddSchemaSvc<T> {
                        type Response = super::Id;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewSchema>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AddSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/AddSchemaVersion" => {
                    #[allow(non_camel_case_types)]
                    struct AddSchemaVersionSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::NewSchemaVersion>
                        for AddSchemaVersionSvc<T>
                    {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewSchemaVersion>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_schema_version(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AddSchemaVersionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/UpdateSchemaMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateSchemaMetadataSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::SchemaMetadataUpdate>
                        for UpdateSchemaMetadataSvc<T>
                    {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaMetadataUpdate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_schema_metadata(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = UpdateSchemaMetadataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/AddViewToSchema" => {
                    #[allow(non_camel_case_types)]
                    struct AddViewToSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::NewSchemaView>
                        for AddViewToSchemaSvc<T>
                    {
                        type Response = super::Id;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewSchemaView>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_view_to_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AddViewToSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/UpdateView" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateViewSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::UpdatedView> for UpdateViewSvc<T> {
                        type Response = super::View;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdatedView>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_view(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = UpdateViewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::VersionedId> for GetSchemaSvc<T> {
                        type Response = super::SchemaDefinition;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::VersionedId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetSchemaMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaMetadataSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetSchemaMetadataSvc<T> {
                        type Response = super::Schema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_metadata(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaMetadataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetSchemaInsertDestination" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaInsertDestinationSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id>
                        for GetSchemaInsertDestinationSvc<T>
                    {
                        type Response = super::SchemaInsertDestination;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_schema_insert_destination(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaInsertDestinationSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetSchemaQueryAddress" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaQueryAddressSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetSchemaQueryAddressSvc<T> {
                        type Response = super::SchemaQueryAddress;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_schema_query_address(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaQueryAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetSchemaVersions" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaVersionsSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetSchemaVersionsSvc<T> {
                        type Response = super::SchemaVersions;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_versions(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaVersionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetSchemaType" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaTypeSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetSchemaTypeSvc<T> {
                        type Response = super::SchemaType;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_type(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaTypeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetView" => {
                    #[allow(non_camel_case_types)]
                    struct GetViewSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetViewSvc<T> {
                        type Response = super::View;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_view(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetViewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetAllSchemas" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllSchemasSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for GetAllSchemasSvc<T> {
                        type Response = super::Schemas;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_all_schemas(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetAllSchemasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetAllSchemaNames" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllSchemaNamesSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for GetAllSchemaNamesSvc<T> {
                        type Response = super::SchemaNames;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_all_schema_names(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetAllSchemaNamesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetAllViewsOfSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllViewsOfSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetAllViewsOfSchemaSvc<T> {
                        type Response = super::SchemaViews;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_all_views_of_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetAllViewsOfSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/GetBaseSchemaOfView" => {
                    #[allow(non_camel_case_types)]
                    struct GetBaseSchemaOfViewSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetBaseSchemaOfViewSvc<T> {
                        type Response = super::ViewSchema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).get_base_schema_of_view(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetBaseSchemaOfViewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/ValidateValue" => {
                    #[allow(non_camel_case_types)]
                    struct ValidateValueSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::ValueToValidate>
                        for ValidateValueSvc<T>
                    {
                        type Response = super::Errors;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValueToValidate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).validate_value(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = ValidateValueSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/PromoteToMaster" => {
                    #[allow(non_camel_case_types)]
                    struct PromoteToMasterSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for PromoteToMasterSvc<T> {
                        type Response = super::PodName;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).promote_to_master(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = PromoteToMasterSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for HeartbeatSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).heartbeat(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = HeartbeatSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: SchemaRegistry> Clone for SchemaRegistryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: SchemaRegistry> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SchemaRegistry> tonic::transport::NamedService for SchemaRegistryServer<T> {
        const NAME: &'static str = "schema_registry.SchemaRegistry";
    }
}
