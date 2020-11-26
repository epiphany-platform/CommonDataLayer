use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct PostgresOutputConfig {
    #[structopt(long, env = "POSTGRES_USERNAME")]
    pub username: String,
    #[structopt(long, env = "POSTGRES_PASSWORD")]
    pub password: String,
    #[structopt(long, env = "POSTGRES_HOST")]
    pub host: String,
    #[structopt(long, env = "POSTGRES_PORT", default_value = "5432")]
    pub port: u16,
    #[structopt(long, env = "POSTGRES_DBNAME")]
    pub dbname: String,
}
