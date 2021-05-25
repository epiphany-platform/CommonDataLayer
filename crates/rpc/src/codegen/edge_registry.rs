#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TreeQuery {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub relations: ::prost::alloc::vec::Vec<TreeQuery>,
    #[prost(string, repeated, tag = "3")]
    pub filter_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TreeResponse {
    #[prost(message, repeated, tag = "1")]
    pub objects: ::prost::alloc::vec::Vec<TreeObject>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TreeObject {
    #[prost(string, required, tag = "1")]
    pub object_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub children: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "4")]
    pub subtrees: ::prost::alloc::vec::Vec<TreeResponse>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaRelation {
    #[prost(string, required, tag = "1")]
    pub parent_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub child_schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationId {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationQuery {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidateRelationQuery {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationResponse {
    #[prost(string, optional, tag = "1")]
    pub child_schema_id: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaId {
    #[prost(string, required, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationDetails {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub child_schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationList {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<RelationDetails>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectRelations {
    #[prost(message, repeated, tag = "1")]
    pub relations: ::prost::alloc::vec::Vec<Edge>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Edge {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_object_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub child_object_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationIdQuery {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_object_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectIdQuery {
    #[prost(string, required, tag = "1")]
    pub object_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
#[doc = r" Generated client implementations."]
pub mod edge_registry_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct EdgeRegistryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EdgeRegistryClient<tonic::transport::Channel> {
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
    impl<T> EdgeRegistryClient<T>
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
        pub async fn add_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::SchemaRelation>,
        ) -> Result<tonic::Response<super::RelationId>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/AddRelation");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::RelationQuery>,
        ) -> Result<tonic::Response<super::RelationResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/GetRelation");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_by_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::RelationId>,
        ) -> Result<tonic::Response<super::SchemaRelation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/edge_registry.EdgeRegistry/GetSchemaByRelation",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_relations(
            &mut self,
            request: impl tonic::IntoRequest<super::SchemaId>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/edge_registry.EdgeRegistry/GetSchemaRelations",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_relations(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/ListRelations");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn validate_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidateRelationQuery>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/edge_registry.EdgeRegistry/ValidateRelation",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_edges(
            &mut self,
            request: impl tonic::IntoRequest<super::ObjectRelations>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/AddEdges");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_edge(
            &mut self,
            request: impl tonic::IntoRequest<super::RelationIdQuery>,
        ) -> Result<tonic::Response<super::Edge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/GetEdge");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_edges(
            &mut self,
            request: impl tonic::IntoRequest<super::ObjectIdQuery>,
        ) -> Result<tonic::Response<super::ObjectRelations>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/GetEdges");
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
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/Heartbeat");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn resolve_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::TreeQuery>,
        ) -> Result<tonic::Response<super::TreeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/ResolveTree");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for EdgeRegistryClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for EdgeRegistryClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "EdgeRegistryClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod edge_registry_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with EdgeRegistryServer."]
    #[async_trait]
    pub trait EdgeRegistry: Send + Sync + 'static {
        async fn add_relation(
            &self,
            request: tonic::Request<super::SchemaRelation>,
        ) -> Result<tonic::Response<super::RelationId>, tonic::Status>;
        async fn get_relation(
            &self,
            request: tonic::Request<super::RelationQuery>,
        ) -> Result<tonic::Response<super::RelationResponse>, tonic::Status>;
        async fn get_schema_by_relation(
            &self,
            request: tonic::Request<super::RelationId>,
        ) -> Result<tonic::Response<super::SchemaRelation>, tonic::Status>;
        async fn get_schema_relations(
            &self,
            request: tonic::Request<super::SchemaId>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status>;
        async fn list_relations(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status>;
        async fn validate_relation(
            &self,
            request: tonic::Request<super::ValidateRelationQuery>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn add_edges(
            &self,
            request: tonic::Request<super::ObjectRelations>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn get_edge(
            &self,
            request: tonic::Request<super::RelationIdQuery>,
        ) -> Result<tonic::Response<super::Edge>, tonic::Status>;
        async fn get_edges(
            &self,
            request: tonic::Request<super::ObjectIdQuery>,
        ) -> Result<tonic::Response<super::ObjectRelations>, tonic::Status>;
        async fn heartbeat(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn resolve_tree(
            &self,
            request: tonic::Request<super::TreeQuery>,
        ) -> Result<tonic::Response<super::TreeResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct EdgeRegistryServer<T: EdgeRegistry> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: EdgeRegistry> EdgeRegistryServer<T> {
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
    impl<T, B> Service<http::Request<B>> for EdgeRegistryServer<T>
    where
        T: EdgeRegistry,
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
                "/edge_registry.EdgeRegistry/AddRelation" => {
                    #[allow(non_camel_case_types)]
                    struct AddRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::SchemaRelation> for AddRelationSvc<T> {
                        type Response = super::RelationId;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaRelation>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AddRelationSvc(inner);
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
                "/edge_registry.EdgeRegistry/GetRelation" => {
                    #[allow(non_camel_case_types)]
                    struct GetRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::RelationQuery> for GetRelationSvc<T> {
                        type Response = super::RelationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelationQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetRelationSvc(inner);
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
                "/edge_registry.EdgeRegistry/GetSchemaByRelation" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaByRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::RelationId> for GetSchemaByRelationSvc<T> {
                        type Response = super::SchemaRelation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelationId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_by_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaByRelationSvc(inner);
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
                "/edge_registry.EdgeRegistry/GetSchemaRelations" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaRelationsSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::SchemaId> for GetSchemaRelationsSvc<T> {
                        type Response = super::RelationList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_relations(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetSchemaRelationsSvc(inner);
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
                "/edge_registry.EdgeRegistry/ListRelations" => {
                    #[allow(non_camel_case_types)]
                    struct ListRelationsSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::Empty> for ListRelationsSvc<T> {
                        type Response = super::RelationList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_relations(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = ListRelationsSvc(inner);
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
                "/edge_registry.EdgeRegistry/ValidateRelation" => {
                    #[allow(non_camel_case_types)]
                    struct ValidateRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::ValidateRelationQuery>
                        for ValidateRelationSvc<T>
                    {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidateRelationQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).validate_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = ValidateRelationSvc(inner);
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
                "/edge_registry.EdgeRegistry/AddEdges" => {
                    #[allow(non_camel_case_types)]
                    struct AddEdgesSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::ObjectRelations> for AddEdgesSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ObjectRelations>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_edges(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = AddEdgesSvc(inner);
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
                "/edge_registry.EdgeRegistry/GetEdge" => {
                    #[allow(non_camel_case_types)]
                    struct GetEdgeSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::RelationIdQuery> for GetEdgeSvc<T> {
                        type Response = super::Edge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelationIdQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_edge(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetEdgeSvc(inner);
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
                "/edge_registry.EdgeRegistry/GetEdges" => {
                    #[allow(non_camel_case_types)]
                    struct GetEdgesSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::ObjectIdQuery> for GetEdgesSvc<T> {
                        type Response = super::ObjectRelations;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ObjectIdQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_edges(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetEdgesSvc(inner);
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
                "/edge_registry.EdgeRegistry/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::Empty> for HeartbeatSvc<T> {
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
                "/edge_registry.EdgeRegistry/ResolveTree" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveTreeSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::TreeQuery> for ResolveTreeSvc<T> {
                        type Response = super::TreeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TreeQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).resolve_tree(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = ResolveTreeSvc(inner);
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
    impl<T: EdgeRegistry> Clone for EdgeRegistryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: EdgeRegistry> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: EdgeRegistry> tonic::transport::NamedService for EdgeRegistryServer<T> {
        const NAME: &'static str = "edge_registry.EdgeRegistry";
    }
}
