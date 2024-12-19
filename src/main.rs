mod command;
mod newtypes;

use std::{error::Error, str::FromStr};

use anchor_client::Cluster;
use clap::Parser;
use command::{App, Command};
use solana_program::pubkey;
use solana_sdk::pubkey::Pubkey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::parse();

    let cluster = app
        .rpc_url
        .map(|address| Cluster::from_str(&address))
        .unwrap_or(Ok(Cluster::Localnet))?;

    let program_id: Pubkey = app
        .program_id
        .map(|x| x.0)
        .unwrap_or(pubkey!("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"));

    match app.command {
        Command::ProgramConfigInit(program_config_init) => {
            program_config_init.execute(cluster, program_id).await?;
        }
        Command::MultisigCreate(multisig_create) => {
            multisig_create.execute(cluster, program_id).await?;
        }
    }
    Ok(())
}
