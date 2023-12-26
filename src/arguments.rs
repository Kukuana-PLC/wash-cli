use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Arguments {


    #[command(subcommand)]
    pub command: Commands,
}

impl Arguments {
    pub fn get_arguments() -> Arguments {
        Arguments::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// runs the current project in development mode with hot reload
    Dev(DevArgs)
}

#[derive(Args, Debug, Clone)]
pub struct DevArgs {
    /// Path to projects wadm.yaml
    pub config: PathBuf,

    // If you want a simple option for just one actor
    // No required until compound dev mode is ready
    // #[arg(long)]
    // pub simple: bool
}



