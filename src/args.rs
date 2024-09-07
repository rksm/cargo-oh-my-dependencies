use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub enum Args {
    #[command(name = "omd")]
    #[command(about, author, version)]
    Omd(Opt),
}

#[derive(Debug, Clone, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Opt {
    #[clap(default_value = "Cargo.toml", help = "Path to Cargo.toml file")]
    pub manifest: PathBuf,
}
