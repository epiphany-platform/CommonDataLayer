#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OnDemandRequest {
    #[prost(string, required, tag = "1")]
    pub view_id: ::prost::alloc::string::String,
    #[prost(map = "string, message", tag = "2")]
    pub schemas: ::std::collections::HashMap<::prost::alloc::string::String, Schema>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Schema {
    #[prost(string, repeated, tag = "1")]
    pub object_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
#[doc = r" Generated client implementations."]
pub mod on_demand_materializer_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct OnDemandMaterializerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl OnDemandMaterializerClient<tonic::transport::Channel> {
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
    impl<T> OnDemandMaterializerClient<T>
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
        pub async fn materialize(
            &mut self,
            request: impl tonic::IntoRequest<super::OnDemandRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::super::common::RowDefinition>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/materializer_ondemand.OnDemandMaterializer/Materialize",
            );
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
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
            let path = http::uri::PathAndQuery::from_static(
                "/materializer_ondemand.OnDemandMaterializer/Heartbeat",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for OnDemandMaterializerClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for OnDemandMaterializerClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "OnDemandMaterializerClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod on_demand_materializer_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with OnDemandMaterializerServer."]
    #[async_trait]
    pub trait OnDemandMaterializer: Send + Sync + 'static {
        #[doc = "Server streaming response type for the Materialize method."]
        type MaterializeStream: futures_core::Stream<Item = Result<super::super::common::RowDefinition, tonic::Status>>
            + Send
            + Sync
            + 'static;
        async fn materialize(
            &self,
            request: tonic::Request<super::OnDemandRequest>,
        ) -> Result<tonic::Response<Self::MaterializeStream>, tonic::Status>;
        async fn heartbeat(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct OnDemandMaterializerServer<T: OnDemandMaterializer> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: OnDemandMaterializer> OnDemandMaterializerServer<T> {
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
    impl<T, B> Service<http::Request<B>> for OnDemandMaterializerServer<T>
    where
        T: OnDemandMaterializer,
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
                "/materializer_ondemand.OnDemandMaterializer/Materialize" => {
                    #[allow(non_camel_case_types)]
                    struct MaterializeSvc<T: OnDemandMaterializer>(pub Arc<T>);
                    impl<T: OnDemandMaterializer>
                        tonic::server::ServerStreamingService<super::OnDemandRequest>
                        for MaterializeSvc<T>
                    {
                        type Response = super::super::common::RowDefinition;
                        type ResponseStream = T::MaterializeStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::OnDemandRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).materialize(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1;
                        let inner = inner.0;
                        let method = MaterializeSvc(inner);
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
                "/materializer_ondemand.OnDemandMaterializer/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: OnDemandMaterializer>(pub Arc<T>);
                    impl<T: OnDemandMaterializer> tonic::server::UnaryService<super::Empty> for HeartbeatSvc<T> {
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
    impl<T: OnDemandMaterializer> Clone for OnDemandMaterializerServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: OnDemandMaterializer> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: OnDemandMaterializer> tonic::transport::NamedService for OnDemandMaterializerServer<T> {
        const NAME: &'static str = "materializer_ondemand.OnDemandMaterializer";
    }
}
