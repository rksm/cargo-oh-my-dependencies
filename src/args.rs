use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, short)]
    pub config_toml: Option<PathBuf>,
}
