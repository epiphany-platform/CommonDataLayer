use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct SleighOutputConfig {
    #[structopt(
        name = "sleigh-addr",
        long = "sleigh-output-addr",
        env = "SLEIGH_OUTPUT_ADDR"
    )]
    pub addr: String,
    #[structopt(
        long = "sleigh-connection-pool-size",
        env = "SLEIGH_CONNECTION_POOL_SIZE",
        default_value = "10"
    )]
    pub pool_size: u32,
}
