fn main() -> std::io::Result<()> {
    tonic_build::configure().compile(
        &[
            "proto/command_service.proto",
            "proto/query_service.proto",
            "proto/query_service_ts.proto",
            "proto/schema_registry.proto",
        ],
        &["proto/"],
    )
}
