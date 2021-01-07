pub enum InputConfig {
    Kafka(KafkaConfig),
    GRpc(GRpcConfig),
}

#[derive(Clone, Debug)]
pub struct KafkaConfig {
    pub group_id: String,
    pub brokers: String,
    pub topic: String,
    pub task_limit: usize,
}

#[derive(Clone, Debug)]
pub struct GRpcConfig {
    pub grpc_port: u16,
}
