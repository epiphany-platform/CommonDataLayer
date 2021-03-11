use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct PostgresOutputConfig {
    /// Postgres username
    #[structopt(long, env = "POSTGRES_USERNAME")]
    pub username: String,
    /// Postgres password
    #[structopt(long, env = "POSTGRES_PASSWORD")]
    pub password: String,
    /// Host of the postgres server
    #[structopt(long, env = "POSTGRES_HOST")]
    pub host: String,
    /// Port on which postgres server listens
    #[structopt(long, env = "POSTGRES_PORT", default_value = "5432")]
    pub port: u16,
    /// Database name
    #[structopt(long, env = "POSTGRES_DBNAME")]
    pub dbname: String,
    /// SQL schema available for service
    #[structopt(long, env = "POSTGRES_SCHEMA", default_value = "public")]
    pub schema: String,
}
