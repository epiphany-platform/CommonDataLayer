#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Range {
    #[prost(string, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub object_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub start: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub end: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub step: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaId {
    #[prost(string, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeSeries {
    #[prost(string, tag = "1")]
    pub timeseries: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawStatement {
    #[prost(string, tag = "1")]
    pub raw_statement: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValueBytes {
    #[prost(bytes = "vec", tag = "1")]
    pub value_bytes: ::prost::alloc::vec::Vec<u8>,
}
#[doc = r" Generated client implementations."]
pub mod query_service_ts_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct QueryServiceTsClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceTsClient<tonic::transport::Channel> {
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
    impl<T> QueryServiceTsClient<T>
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
        pub async fn query_by_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::SchemaId>,
        ) -> Result<tonic::Response<super::TimeSeries>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/query_service_ts.QueryServiceTs/QueryBySchema",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn query_by_range(
            &mut self,
            request: impl tonic::IntoRequest<super::Range>,
        ) -> Result<tonic::Response<super::TimeSeries>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/query_service_ts.QueryServiceTs/QueryByRange",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn query_raw(
            &mut self,
            request: impl tonic::IntoRequest<super::RawStatement>,
        ) -> Result<tonic::Response<super::ValueBytes>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/query_service_ts.QueryServiceTs/QueryRaw");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for QueryServiceTsClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for QueryServiceTsClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "QueryServiceTsClient {{ ... }}")
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod query_service_ts_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with QueryServiceTsServer."]
    #[async_trait]
    pub trait QueryServiceTs: Send + Sync + 'static {
        async fn query_by_schema(
            &self,
            request: tonic::Request<super::SchemaId>,
        ) -> Result<tonic::Response<super::TimeSeries>, tonic::Status>;
        async fn query_by_range(
            &self,
            request: tonic::Request<super::Range>,
        ) -> Result<tonic::Response<super::TimeSeries>, tonic::Status>;
        async fn query_raw(
            &self,
            request: tonic::Request<super::RawStatement>,
        ) -> Result<tonic::Response<super::ValueBytes>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct QueryServiceTsServer<T: QueryServiceTs> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: QueryServiceTs> QueryServiceTsServer<T> {
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
    impl<T, B> Service<http::Request<B>> for QueryServiceTsServer<T>
    where
        T: QueryServiceTs,
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
                "/query_service_ts.QueryServiceTs/QueryBySchema" => {
                    #[allow(non_camel_case_types)]
                    struct QueryBySchemaSvc<T: QueryServiceTs>(pub Arc<T>);
                    impl<T: QueryServiceTs> tonic::server::UnaryService<super::SchemaId> for QueryBySchemaSvc<T> {
                        type Response = super::TimeSeries;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).query_by_schema(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = QueryBySchemaSvc(inner);
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
                "/query_service_ts.QueryServiceTs/QueryByRange" => {
                    #[allow(non_camel_case_types)]
                    struct QueryByRangeSvc<T: QueryServiceTs>(pub Arc<T>);
                    impl<T: QueryServiceTs> tonic::server::UnaryService<super::Range> for QueryByRangeSvc<T> {
                        type Response = super::TimeSeries;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Range>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).query_by_range(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = QueryByRangeSvc(inner);
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
                "/query_service_ts.QueryServiceTs/QueryRaw" => {
                    #[allow(non_camel_case_types)]
                    struct QueryRawSvc<T: QueryServiceTs>(pub Arc<T>);
                    impl<T: QueryServiceTs> tonic::server::UnaryService<super::RawStatement> for QueryRawSvc<T> {
                        type Response = super::ValueBytes;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RawStatement>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).query_raw(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = QueryRawSvc(inner);
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
    impl<T: QueryServiceTs> Clone for QueryServiceTsServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: QueryServiceTs> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: QueryServiceTs> tonic::transport::NamedService for QueryServiceTsServer<T> {
        const NAME: &'static str = "query_service_ts.QueryServiceTs";
    }
}
