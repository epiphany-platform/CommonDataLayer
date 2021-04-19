use clap::Clap;

#[derive(Clone, Debug, Clap)]
pub struct PostgresOutputConfig {
    /// Postgres username
    #[clap(long, env = "POSTGRES_USERNAME")]
    pub username: String,
    /// Postgres password
    #[clap(long, env = "POSTGRES_PASSWORD")]
    pub password: String,
    /// Host of the postgres server
    #[clap(long, env = "POSTGRES_HOST")]
    pub host: String,
    /// Port on which postgres server listens
    #[clap(long, env = "POSTGRES_PORT", default_value = "5432")]
    pub port: u16,
    /// Database name
    #[clap(long, env = "POSTGRES_DBNAME")]
    pub dbname: String,
    /// SQL schema available for service
    #[clap(long, env = "POSTGRES_SCHEMA", default_value = "public")]
    pub schema: String,
}
