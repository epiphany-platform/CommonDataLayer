use std::sync::Arc;

use crate::{config::Config, error::Error};
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;

pub struct Context {
    config: Arc<Config>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(config: Arc<Config>) -> Self {
        Context { config }
    }

    pub async fn connect_to_registry(&self) -> Result<SchemaRegistryConn, Error> {
        // TODO: Make proper connection pool
        let new_conn = rpc::schema_registry::connect(self.config.registry_addr.clone()).await?;
        Ok(new_conn)
    }
}

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;
