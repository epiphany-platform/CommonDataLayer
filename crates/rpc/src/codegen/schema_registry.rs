#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewSchema {
    #[prost(message, required, tag = "1")]
    pub metadata: SchemaMetadata,
    #[prost(bytes = "vec", required, tag = "2")]
    pub definition: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaMetadata {
    #[prost(string, required, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub insert_destination: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub query_address: ::prost::alloc::string::String,
    #[prost(message, required, tag = "4")]
    pub schema_type: SchemaType,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaMetadataPatch {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub query_address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub insert_destination: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, optional, tag = "4")]
    pub schema_type: ::core::option::Option<SchemaType>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schema {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, required, tag = "2")]
    pub metadata: SchemaMetadata,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct View {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub materializer_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "5")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "6")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "7")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Filter {
    #[prost(oneof = "filter::FilterKind", tags = "1, 2")]
    pub filter_kind: ::core::option::Option<filter::FilterKind>,
}
/// Nested message and enum types in `Filter`.
pub mod filter {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FilterKind {
        #[prost(message, tag = "1")]
        Simple(super::SimpleFilter),
        #[prost(message, tag = "2")]
        Complex(super::ComplexFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleFilter {
    #[prost(message, required, tag = "1")]
    pub operator: FilterOperator,
    #[prost(message, required, tag = "2")]
    pub lhs: FilterValue,
    #[prost(message, optional, tag = "3")]
    pub rhs: ::core::option::Option<FilterValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComplexFilter {
    #[prost(message, required, tag = "1")]
    pub operator: LogicOperator,
    #[prost(message, repeated, tag = "2")]
    pub operands: ::prost::alloc::vec::Vec<Filter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogicOperator {
    #[prost(enumeration = "logic_operator::Operator", required, tag = "1")]
    pub operator: i32,
}
/// Nested message and enum types in `LogicOperator`.
pub mod logic_operator {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Operator {
        And = 0,
        Or = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FilterOperator {
    #[prost(enumeration = "filter_operator::Operator", required, tag = "1")]
    pub operator: i32,
}
/// Nested message and enum types in `FilterOperator`.
pub mod filter_operator {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Operator {
        EqualsOp = 0,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FilterValue {
    #[prost(oneof = "filter_value::FilterValue", tags = "1, 2, 3, 4")]
    pub filter_value: ::core::option::Option<filter_value::FilterValue>,
}
/// Nested message and enum types in `FilterValue`.
pub mod filter_value {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FilterValue {
        #[prost(message, tag = "1")]
        SchemaField(super::SchemaFieldFilter),
        #[prost(message, tag = "2")]
        ViewPath(super::ViewPathFilter),
        #[prost(message, tag = "3")]
        RawValue(super::RawValueFilter),
        #[prost(message, tag = "4")]
        Computed(super::ComputedFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaFieldFilter {
    #[prost(uint32, required, tag = "1")]
    pub schema_id: u32,
    #[prost(string, required, tag = "2")]
    pub field_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewPathFilter {
    #[prost(string, required, tag = "1")]
    pub field_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawValueFilter {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComputedFilter {
    #[prost(message, required, tag = "1")]
    pub computation: Computation,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Computation {
    #[prost(message, required, tag = "1")]
    pub operator: ComputationOperator,
    #[prost(message, required, boxed, tag = "2")]
    pub lhs: ::prost::alloc::boxed::Box<Computation>,
    #[prost(message, optional, boxed, tag = "3")]
    pub rhs: ::core::option::Option<::prost::alloc::boxed::Box<Computation>>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComputationOperator {
    #[prost(oneof = "computation_operator::ComputationOperator", tags = "1, 2, 3")]
    pub computation_operator: ::core::option::Option<computation_operator::ComputationOperator>,
}
/// Nested message and enum types in `ComputationOperator`.
pub mod computation_operator {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum ComputationOperator {
        #[prost(message, tag = "1")]
        RawValue(super::RawValueComputation),
        #[prost(message, tag = "2")]
        FieldValue(super::FieldValueComputation),
        #[prost(message, tag = "3")]
        EqualsComputation(super::EqualsComputation),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawValueComputation {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldValueComputation {
    #[prost(string, optional, tag = "1")]
    pub base_schema: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, required, tag = "2")]
    pub field_path: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EqualsComputation {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullView {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub base_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "5")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "6")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "7")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "8")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewView {
    #[prost(string, required, tag = "1")]
    pub base_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub materializer_address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "5")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(message, optional, tag = "6")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "7")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Relation {
    #[prost(string, required, tag = "1")]
    pub global_id: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "2")]
    pub local_id: u32,
    #[prost(message, required, tag = "3")]
    pub search_for: SearchFor,
    #[prost(message, repeated, tag = "4")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchFor {
    #[prost(enumeration = "search_for::Direction", required, tag = "1")]
    pub search_for: i32,
}
/// Nested message and enum types in `SearchFor`.
pub mod search_for {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Direction {
        Parents = 0,
        Children = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ViewUpdate {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub materializer_address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, required, tag = "4")]
    pub materializer_options: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "5")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    #[prost(bool, required, tag = "6")]
    pub update_fields: bool,
    #[prost(message, optional, tag = "7")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "8")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
    #[prost(bool, required, tag = "9")]
    pub update_filters: bool,
    #[prost(bool, required, tag = "10")]
    pub update_relations: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullSchema {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, required, tag = "2")]
    pub metadata: SchemaMetadata,
    #[prost(message, repeated, tag = "3")]
    pub definitions: ::prost::alloc::vec::Vec<SchemaDefinition>,
    #[prost(message, repeated, tag = "4")]
    pub views: ::prost::alloc::vec::Vec<View>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewSchemaVersion {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, required, tag = "2")]
    pub definition: SchemaDefinition,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaMetadataUpdate {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, required, tag = "2")]
    pub patch: SchemaMetadataPatch,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionedId {
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub version_req: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaDefinition {
    #[prost(string, required, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(bytes = "vec", required, tag = "2")]
    pub definition: ::prost::alloc::vec::Vec<u8>,
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
    #[prost(message, repeated, tag = "1")]
    pub schemas: ::prost::alloc::vec::Vec<Schema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FullSchemas {
    #[prost(message, repeated, tag = "1")]
    pub schemas: ::prost::alloc::vec::Vec<FullSchema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaViews {
    #[prost(message, repeated, tag = "1")]
    pub views: ::prost::alloc::vec::Vec<FullView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueToValidate {
    #[prost(message, required, tag = "1")]
    pub schema_id: VersionedId,
    #[prost(bytes = "vec", required, tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Errors {
    #[prost(string, repeated, tag = "1")]
    pub errors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
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
        pub async fn update_schema(
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
                "/schema_registry.SchemaRegistry/UpdateSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_view_to_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::NewView>,
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
            request: impl tonic::IntoRequest<super::ViewUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
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
        pub async fn get_schema_metadata(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::SchemaMetadata>, tonic::Status> {
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
        pub async fn get_schema_definition(
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
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetSchemaDefinition",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_full_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::FullSchema>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetFullSchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_view(
            &mut self,
            request: impl tonic::IntoRequest<super::Id>,
        ) -> Result<tonic::Response<super::FullView>, tonic::Status> {
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
        pub async fn get_all_full_schemas(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::FullSchemas>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/GetAllFullSchemas",
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
        ) -> Result<tonic::Response<super::Schema>, tonic::Status> {
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
        pub async fn watch_all_schema_updates(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::Schema>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/schema_registry.SchemaRegistry/WatchAllSchemaUpdates",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn ping(
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
            let path = http::uri::PathAndQuery::from_static("/schema_registry.SchemaRegistry/Ping");
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
        async fn update_schema(
            &self,
            request: tonic::Request<super::SchemaMetadataUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn add_view_to_schema(
            &self,
            request: tonic::Request<super::NewView>,
        ) -> Result<tonic::Response<super::Id>, tonic::Status>;
        async fn update_view(
            &self,
            request: tonic::Request<super::ViewUpdate>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn get_schema_metadata(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaMetadata>, tonic::Status>;
        async fn get_schema_versions(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaVersions>, tonic::Status>;
        async fn get_schema_definition(
            &self,
            request: tonic::Request<super::VersionedId>,
        ) -> Result<tonic::Response<super::SchemaDefinition>, tonic::Status>;
        async fn get_full_schema(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::FullSchema>, tonic::Status>;
        async fn get_view(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::FullView>, tonic::Status>;
        async fn get_all_schemas(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Schemas>, tonic::Status>;
        async fn get_all_full_schemas(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::FullSchemas>, tonic::Status>;
        async fn get_all_views_of_schema(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::SchemaViews>, tonic::Status>;
        async fn get_base_schema_of_view(
            &self,
            request: tonic::Request<super::Id>,
        ) -> Result<tonic::Response<super::Schema>, tonic::Status>;
        async fn validate_value(
            &self,
            request: tonic::Request<super::ValueToValidate>,
        ) -> Result<tonic::Response<super::Errors>, tonic::Status>;
        #[doc = "Server streaming response type for the WatchAllSchemaUpdates method."]
        type WatchAllSchemaUpdatesStream: futures_core::Stream<Item = Result<super::Schema, tonic::Status>>
            + Send
            + Sync
            + 'static;
        async fn watch_all_schema_updates(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<Self::WatchAllSchemaUpdatesStream>, tonic::Status>;
        async fn ping(
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
                "/schema_registry.SchemaRegistry/UpdateSchema" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::SchemaMetadataUpdate>
                        for UpdateSchemaSvc<T>
                    {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaMetadataUpdate>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).update_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = UpdateSchemaSvc(inner);
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
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::NewView> for AddViewToSchemaSvc<T> {
                        type Response = super::Id;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NewView>,
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
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::ViewUpdate> for UpdateViewSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ViewUpdate>,
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
                "/schema_registry.SchemaRegistry/GetSchemaMetadata" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaMetadataSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetSchemaMetadataSvc<T> {
                        type Response = super::SchemaMetadata;
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
                "/schema_registry.SchemaRegistry/GetSchemaDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaDefinitionSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::VersionedId>
                        for GetSchemaDefinitionSvc<T>
                    {
                        type Response = super::SchemaDefinition;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::VersionedId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_definition(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaDefinitionSvc(inner);
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
                "/schema_registry.SchemaRegistry/GetFullSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetFullSchemaSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Id> for GetFullSchemaSvc<T> {
                        type Response = super::FullSchema;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Id>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_full_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetFullSchemaSvc(inner);
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
                        type Response = super::FullView;
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
                "/schema_registry.SchemaRegistry/GetAllFullSchemas" => {
                    #[allow(non_camel_case_types)]
                    struct GetAllFullSchemasSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for GetAllFullSchemasSvc<T> {
                        type Response = super::FullSchemas;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_all_full_schemas(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetAllFullSchemasSvc(inner);
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
                        type Response = super::Schema;
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
                "/schema_registry.SchemaRegistry/WatchAllSchemaUpdates" => {
                    #[allow(non_camel_case_types)]
                    struct WatchAllSchemaUpdatesSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::ServerStreamingService<super::Empty>
                        for WatchAllSchemaUpdatesSvc<T>
                    {
                        type Response = super::Schema;
                        type ResponseStream = T::WatchAllSchemaUpdatesStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).watch_all_schema_updates(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1;
                        let inner = inner.0;
                        let method = WatchAllSchemaUpdatesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/schema_registry.SchemaRegistry/Ping" => {
                    #[allow(non_camel_case_types)]
                    struct PingSvc<T: SchemaRegistry>(pub Arc<T>);
                    impl<T: SchemaRegistry> tonic::server::UnaryService<super::Empty> for PingSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).ping(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = PingSvc(inner);
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
