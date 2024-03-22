use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap()]
    pub config_toml: Option<PathBuf>,
}
