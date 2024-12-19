pub mod multisig_create;
pub mod program_config_init;

use clap::{Parser, Subcommand};
use multisig_create::MultisigCreate;
use program_config_init::ProgramConfigInit;

use crate::newtypes::ClapAddress;

#[derive(Parser)]
pub struct App {
    #[arg(long)]
    pub rpc_url: Option<String>,
    #[arg(long)]
    pub program_id: Option<ClapAddress>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initializes the program config
    ProgramConfigInit(ProgramConfigInit),
    /// Creates a multisig account
    MultisigCreate(MultisigCreate),
}
