use anyhow::Context;
use futures::{Future, FutureExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{DeleteParams, ListParams, Meta, PatchParams},
    Api, Client,
};
use log::{info, warn};
use serde::Deserialize;
use std::{fs, time::Duration};

#[derive(Deserialize)]
pub struct Config {
    pub heartbeat_secs: u64,
    pub schema_app_name: String,
    pub schema_addr: String,
    pub schema_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = envy::from_env::<Config>().context("Env vars not set correctly")?;
    let namespace = get_k8s_namespace();
    let schema_elector = LeaderElector {
        master_addr: format!("{}-master", config.schema_addr),
        slave_addr: config.schema_addr,
        heartbeat_time: Duration::from_secs(config.heartbeat_secs),
        name: config.schema_app_name,
        election_type: LeaderElectorType::Schema,
        port: config.schema_port,
        namespace,
    };

    schema_elector.elect_leader().await
}

#[derive(Debug)]
pub enum LeaderElectorType {
    Schema,
}

struct LeaderElector {
    master_addr: String,
    slave_addr: String,
    name: String,
    port: u16,
    heartbeat_time: Duration,
    election_type: LeaderElectorType,
    namespace: String,
}

impl LeaderElector {
    pub async fn elect_leader(&self) -> anyhow::Result<()> {
        let client_api = Client::try_default().await?;
        let pods: Api<Pod> = Api::namespaced(client_api, &self.namespace);
        loop {
            tokio::time::delay_for(self.heartbeat_time).await;

            let heartbeat = match self.election_type {
                LeaderElectorType::Schema => schema_heartbeat(&self.master_addr, self.port).await,
            };

            if heartbeat.is_err() {
                self.remove_master_instance(&pods).await?;
                self.promote_slave_to_master(&pods).await?;
            }
        }
    }

    async fn remove_master_instance(&self, pods: &Api<Pod>) -> anyhow::Result<()> {
        let params = ListParams::default()
            .labels(&format!("app={},role=master", self.name))
            .timeout(10);
        for pod in pods.list(&params).await? {
            let pod_name = Meta::name(&pod);
            let params = DeleteParams {
                grace_period_seconds: Some(0),
                ..DeleteParams::default()
            };
            pods.delete(&pod_name, &params).await?;
            info!("Pod {} marked for deletion", pod_name);
        }
        Ok(())
    }

    async fn promote_slave_to_master(&self, pods: &Api<Pod>) -> anyhow::Result<()> {
        let promotion = match self.election_type {
            LeaderElectorType::Schema => schema_promotion(&self.slave_addr, self.port).await,
        };

        let pod_name = match promotion {
            Ok(name) => name,
            Err(_err) => {
                warn!("No active {:?} pods found", self.election_type);
                return Ok(());
            }
        };
        info!("Promoting pod {}", pod_name);
        let patch = serde_yaml::to_vec(&serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "labels": {
                    "role":"master"
                }
            }
        }))?;
        pods.patch(&pod_name, &PatchParams::apply("Leader-elector"), patch)
            .await?;
        Ok(())
    }
}

fn schema_heartbeat(addr: &str, port: u16) -> impl Future<Output = anyhow::Result<()>> + '_ {
    schema_registry::heartbeat(format!("{}:{}", addr, port)).map(|x| x.map_err(anyhow::Error::new))
}

fn schema_promotion(addr: &str, port: u16) -> impl Future<Output = anyhow::Result<String>> + '_ {
    schema_registry::promote_to_master(format!("{}:{}", addr, port))
        .map(|x| x.map_err(anyhow::Error::new))
}

fn get_k8s_namespace() -> String {
    fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace")
        .expect("Not running in k8s environment")
}
