fn main() -> std::io::Result<()> {
    tonic_build::configure().compile(
        &[
            "proto/blob_storage.proto",
            "proto/command_service.proto",
            "proto/document_storage.proto",
            "proto/query_service.proto",
            "proto/query_service_ts.proto",
            "proto/schema_registry.proto",
        ],
        &["proto/"],
    )
}
