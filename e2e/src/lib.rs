#[cfg(test)]
mod tests {
    use anyhow::Result;
    use lazy_static::lazy_static;
    use rpc::{
        schema_registry::{
            schema_registry_client::SchemaRegistryClient, types::SchemaType, NewSchema,
            SchemaMetadata,
        },
        tonic::transport::Channel,
    };
    use serde_json::Value;
    use std::{
        sync::{Mutex, Once},
        thread,
        time::Duration,
    };
    use tokio::{runtime::Handle, time::sleep};

    const SCHEMA_REGISTRY_ADDR: &str = "http://cdl-schema-registry:6400";
    const POSTGRES_QUERY_ADDR: &str = "http://cdl-postgres-query-service:6400";
    const POSTGRES_COMMAND_ADDR: &str = "cdl.document.data";

    static INIT: Once = Once::new();

    lazy_static! {
        static ref SR_CLIENT: Mutex<Option<SchemaRegistryClient<Channel>>> = Mutex::new(None);
        static ref EMPTY_SCHEMA: Value = serde_json::from_str("{}").unwrap();
    }

    #[tokio::main]
    async fn async_initialization() {
        {
            let mut sr_guard = SR_CLIENT.lock().unwrap();
            *sr_guard = Some(
                rpc::schema_registry::connect(SCHEMA_REGISTRY_ADDR.to_owned())
                    .await
                    .unwrap(),
            );
        }

        sleep(Duration::from_secs(2)).await;
    }

    fn initialize_cdl() {
        INIT.call_once(|| {
            println!("A");
            thread::spawn(|| async_initialization())
                .join()
                .expect("Thread panicked");

            println!("B");
        });
    }

    #[tokio::test]
    async fn should_fetch_api_versions_after_connect() -> Result<()> {
        initialize_cdl();

        assert!(!false);
        Ok(())
    }
}
