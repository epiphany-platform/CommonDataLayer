use crate::{
    db::SchemaDb,
    types::storage::vertices::View,
    types::ViewUpdate,
    types::{NewSchema, NewSchemaVersion},
    AmqpConfig, KafkaConfig,
};
use rpc::schema_registry::types::SchemaType;
use serde::{Deserialize, Serialize};
use std::{
    sync::{mpsc, Arc},
    thread,
};
use structopt::clap::arg_enum;
use tokio::{runtime::Handle, sync::oneshot};
use tracing::info;
use uuid::Uuid;

mod master;
mod slave;

#[derive(Debug, Serialize, Deserialize)]
pub enum ReplicationEvent {
    AddSchema {
        schema: NewSchema,
        id: Uuid,
    },
    AddSchemaVersion {
        id: Uuid,
        new_version: NewSchemaVersion,
    },
    UpdateSchemaMetadata {
        id: Uuid,
        name: Option<String>,
        topic: Option<String>,
        query_address: Option<String>,
        schema_type: Option<SchemaType>,
    },
    AddViewToSchema {
        schema_id: Uuid,
        view: View,
        view_id: Uuid,
    },
    UpdateView {
        id: Uuid,
        view: ViewUpdate,
    },
}

arg_enum! {
    #[derive(Clone, Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum ReplicationRole {
        Master,
        Slave,
        None,
    }
}

pub struct ReplicationState {
    replication_role: ReplicationRole,
    stop_channel: Option<oneshot::Sender<()>>,
    master_replication_channel: Option<mpsc::Sender<ReplicationEvent>>,
    message_queue_config: ReplicationMethodConfig,
    db: Arc<SchemaDb>,
}
impl ReplicationState {
    pub fn new(
        message_queue_config: ReplicationMethodConfig,
        db: Arc<SchemaDb>,
    ) -> ReplicationState {
        ReplicationState {
            replication_role: ReplicationRole::None,
            stop_channel: None,
            master_replication_channel: None,
            message_queue_config,
            db,
        }
    }

    pub fn set_role(&mut self, role: ReplicationRole) {
        // send signal to clear old replication, if any
        // old replication resources will be destroyed on next message on replication channel
        if let Some(old_channel) = self.stop_channel.take() {
            old_channel.send(()).unwrap(); // TODO: PK: should abort whole app?
        }
        self.master_replication_channel.take();

        self.replication_role = role;
        match self.replication_role {
            ReplicationRole::Master => {
                info!("Replicating as master");
                let (sender, receiver) = oneshot::channel::<()>();
                self.stop_channel = Some(sender);
                self.start_replication_master(receiver);
            }
            ReplicationRole::Slave => {
                info!("Replicating as slave");
                let (sender, receiver) = oneshot::channel::<()>();
                self.stop_channel = Some(sender);
                start_replication_slave(self.db.clone(), &self.message_queue_config, receiver);
            }
            ReplicationRole::None => info!("Replication disabled"),
        }
    }
    pub fn replicate_message(&self, event: ReplicationEvent) {
        if self.replication_role != ReplicationRole::Master {
            return;
        }
        if let Some(sender) = &self.master_replication_channel {
            let result = sender.send(event);
            if result.is_err() {
                info!("Master replication disabled");
                return;
            };
        }
    }
    fn start_replication_master(&mut self, kill_signal: oneshot::Receiver<()>) {
        let tokio_runtime = Handle::current();
        let (send, recv) = mpsc::channel::<ReplicationEvent>();

        let config = self.message_queue_config.clone();
        self.master_replication_channel = Some(send);
        thread::spawn(move || {
            master::replicate_db_events(config, recv, tokio_runtime, kill_signal)
        });
        info!("Replication started as master node.");
    }
}

#[derive(Clone, Debug)]
pub struct ReplicationMethodConfig {
    pub queue: CommunicationMethod,
    pub source: String,
    pub destination: String,
}

#[derive(Clone, Debug)]
pub enum CommunicationMethod {
    Kafka(KafkaConfig),
    Amqp(AmqpConfig),
}

fn start_replication_slave(
    db: Arc<SchemaDb>,
    config: &ReplicationMethodConfig,
    kill_signal: oneshot::Receiver<()>,
) {
    tokio::spawn(slave::consume_mq(config.clone(), db, kill_signal));
    info!("Replication started as slave node.");
}
