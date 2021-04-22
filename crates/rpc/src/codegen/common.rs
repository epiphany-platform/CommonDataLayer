#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RowDefinition {
    #[prost(string, required, tag = "1")]
    pub object_id: ::prost::alloc::string::String,
    #[prost(map = "string, string", tag = "2")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
