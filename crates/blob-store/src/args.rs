use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct Args {
    #[structopt(
        short = "p",
        long,
        env = "DATASTORE_ROOT",
        default_value = "/var/data/blob_store"
    )]
    pub datastore_root: PathBuf,
}
