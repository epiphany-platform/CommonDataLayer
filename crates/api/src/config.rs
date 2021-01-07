use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(long = "schema-registry-addr", env = "SCHEMA_REGISTRY_ADDR")]
    pub registry_addr: String,
    #[structopt(long = "query-router-addr", env = "QUERY_ROUTER_ADDR")]
    pub query_router_addr: String,
    #[structopt(long = "input-port", env = "INPUT_PORT")]
    pub input_port: u16,
}
