#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MaterializedView {
    #[prost(string, required, tag = "1")]
    pub view_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub options: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub rows: ::prost::alloc::vec::Vec<RowDefinition>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RowDefinition {
    #[prost(string, required, tag = "1")]
    pub object_id: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "2")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
