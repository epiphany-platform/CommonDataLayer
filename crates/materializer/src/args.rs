use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Args {
    /// Port to listen on
    #[structopt(long, env)]
    pub input_port: u16,
    /// Port to listen on for Prometheus requests
    #[structopt(long, default_value = utils::metrics::DEFAULT_PORT, env)]
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[structopt(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,

    #[structopt(flatten)]
    pub materializer: MaterializerArgs,
}

#[derive(StructOpt, Debug)]
pub enum MaterializerArgs {
    Postgres(PostgresMaterializerArgs),
}

#[derive(StructOpt, Debug)]
pub struct PostgresMaterializerArgs {
    /// Postgres username
    #[structopt(long, env)]
    pub postgres_username: String,
    /// Postgres password
    #[structopt(long, env)]
    pub postgres_password: String,
    /// Host of the postgres server
    #[structopt(long, env)]
    pub postgres_host: String,
    /// Port on which postgres server listens
    #[structopt(long, env, default_value = "5432")]
    pub postgres_port: u16,
    /// Database name
    #[structopt(long, env)]
    pub postgres_dbname: String,
    /// SQL schema available for service
    #[structopt(long, env, default_value = "public")]
    pub postgres_schema: String,
}
