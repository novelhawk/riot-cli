mod datastore;
mod models;
mod val_store;

use clap::{command, Parser, Subcommand};
use val_store::handle_val_store_command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    pub module: Module,
}

#[derive(Subcommand, Debug)]
enum Module {
    ValStore {
        #[command(subcommand)]
        action: ValStoreCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ValStoreCommands {
    Add,
    Check,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.module {
        Module::ValStore { action } => handle_val_store_command(action).await,
    };
}
