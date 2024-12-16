use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Initializes the program config")]
    ProgramConfigInit {
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        program_id: Option<ClapAddress>,
        #[arg(long)]
        initializer_keypair: ClapKeypair,
        #[arg(long)]
        program_config_authority: ClapAddress,
        #[arg(long)]
        treasury: ClapAddress,
    },
    #[command(about = "Creates a multisig account")]
    MultisigCreate {
        #[arg(long)]
        rpc_url: Option<String>,
        #[arg(long)]
        program_id: Option<ClapAddress>,
        #[arg(long)]
        keypair: ClapKeypair,
        #[arg(long)]
        config_authority: Option<ClapAddress>,
        #[arg(long)]
        rent_collector: Option<ClapAddress>,
        #[arg(long, short, value_delimiter = ' ')]
        members: Vec<String>,
        #[arg(long)]
        threshold: u16,
        #[arg(long)]
        priority_fee_lamports: Option<u64>,
    },
}

#[derive(Clone)]
pub struct ClapKeypair(pub String);

impl From<String> for ClapKeypair {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct ClapAddress(pub String);

impl From<String> for ClapAddress {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::parse();

    match app.command {
        Command::ProgramConfigInit {
            rpc_url,
            program_id,
            initializer_keypair,
            program_config_authority,
            treasury,
        } => {}
        Command::MultisigCreate {
            rpc_url,
            program_id,
            keypair,
            config_authority,
            rent_collector,
            members,
            threshold,
            priority_fee_lamports,
        } => {}
    }
    Ok(())
}
