pub mod multisig_create;
pub mod program_config_init;

use std::{str::FromStr, sync::Arc};

use clap::{Parser, Subcommand};
use multisig_create::MultisigCreate;
use program_config_init::ProgramConfigInit;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::EncodableKey};

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

#[derive(Clone)]
pub struct ClapKeypair(pub Arc<Keypair>);

impl FromStr for ClapKeypair {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keypair =
            Keypair::read_from_file(s).map_err(|e| format!("Error loading keypair: {e}"))?;
        Ok(Self(Arc::new(keypair)))
    }
}

#[derive(Clone, Copy)]
pub struct ClapAddress(pub Pubkey);

impl FromStr for ClapAddress {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pubkey = Pubkey::from_str(s).map_err(|e| format!("Error loading pubkey: {e}"))?;
        Ok(Self(pubkey))
    }
}
