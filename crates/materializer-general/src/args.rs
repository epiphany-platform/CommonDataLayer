use clap::Clap;

#[derive(Clap, Debug)]
pub struct Args {
    /// Port to listen on
    #[clap(long, env)]
    pub input_port: u16,
    /// Port to listen on for Prometheus requests
    #[clap(long, default_value = metrics_utils::DEFAULT_PORT, env)]
    pub metrics_port: u16,
    /// Port exposing status of the application
    #[clap(long, default_value = utils::status_endpoints::DEFAULT_PORT, env)]
    pub status_port: u16,

    #[clap(flatten)]
    pub materializer: MaterializerArgs,
}

#[derive(Clap, Debug)]
pub enum MaterializerArgs {
    Postgres(PostgresMaterializerArgs),
}

#[derive(Clap, Debug)]
pub struct PostgresMaterializerArgs {
    /// Postgres username
    #[clap(long, env)]
    pub postgres_username: String,
    /// Postgres password
    #[clap(long, env)]
    pub postgres_password: String,
    /// Host of the postgres server
    #[clap(long, env)]
    pub postgres_host: String,
    /// Port on which postgres server listens
    #[clap(long, env, default_value = "5432")]
    pub postgres_port: u16,
    /// Database name
    #[clap(long, env)]
    pub postgres_dbname: String,
    /// SQL schema available for service
    #[clap(long, env, default_value = "public")]
    pub postgres_schema: String,
}
