use std::{error::Error, str::FromStr, sync::Arc};

use anchor_client::{
    anchor_lang::{system_program, InstructionData, ToAccountMetas},
    Client, Cluster,
};
use clap::{Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    message::v0::Message,
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
    transaction::VersionedTransaction,
};
use squads_multisig::{
    pda::get_program_config_pda, squads_multisig_program::ProgramConfigInitArgs,
};

#[derive(Parser)]
struct App {
    #[arg(long)]
    rpc_url: Option<String>,
    #[arg(long)]
    program_id: Option<ClapAddress>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Initializes the program config")]
    ProgramConfigInit {
        #[arg(long)]
        initializer_keypair: ClapKeypair,
        #[arg(long)]
        program_config_authority: ClapAddress,
        #[arg(long)]
        treasury: ClapAddress,
        #[arg(long)]
        multisig_creation_fee: u64,
        #[arg(long)]
        priority_fee_lamports: Option<u64>,
    },
    #[command(about = "Creates a multisig account")]
    MultisigCreate {
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
pub struct ClapKeypair(pub Arc<Keypair>);

impl From<&str> for ClapKeypair {
    fn from(value: &str) -> Self {
        let keypair = Keypair::read_from_file(value).unwrap();
        Self(Arc::new(keypair))
    }
}

#[derive(Clone, Copy)]
pub struct ClapAddress(pub Pubkey);

impl From<&str> for ClapAddress {
    fn from(value: &str) -> Self {
        let pubkey = Pubkey::from_str(value).unwrap();
        Self(pubkey)
    }
}

async fn program_config_init(
    rpc_client: &RpcClient,
    program_id: Pubkey,
    payer: &Keypair,
    priority_fee_lamports: Option<u64>,
    program_config_authority: Pubkey,
    multisig_creation_fee: u64,
    treasury: Pubkey,
) -> Result<(), Box<dyn Error>> {
    let blockhash = rpc_client.get_latest_blockhash().await?;

    let program_config = get_program_config_pda(Some(&program_id)).0;

    let instructions = &[
        ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports.unwrap_or(5000)),
        Instruction {
            accounts: squads_multisig::squads_multisig_program::accounts::ProgramConfigInit {
                program_config,
                initializer: payer.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(Some(false)),
            data: squads_multisig::squads_multisig_program::instruction::ProgramConfigInit {
                args: ProgramConfigInitArgs {
                    authority: program_config_authority,
                    multisig_creation_fee,
                    treasury,
                },
            }
            .data(),
            program_id,
        },
    ];

    let message = Message::try_compile(&payer.pubkey(), instructions, &[], blockhash)?;

    let transaction = VersionedTransaction::try_new(
        solana_sdk::message::VersionedMessage::V0(message),
        &[&payer],
    )?;

    let signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .await?;

    println!("Program config: {}", program_config);
    println!("Transaction signature: {}", signature);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::parse();

    let cluster = match app.rpc_url {
        Some(address) => Cluster::Custom(address.clone(), address),
        None => Cluster::Localnet,
    };

    let program_id = app
        .program_id
        .map(|x| x.0)
        .unwrap_or(pubkey!("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"));

    match app.command {
        Command::ProgramConfigInit {
            initializer_keypair,
            program_config_authority,
            treasury,
            multisig_creation_fee,
            priority_fee_lamports,
        } => {
            let rpc_client = RpcClient::new(cluster.url().to_string());
            program_config_init(
                &rpc_client,
                program_id,
                &initializer_keypair.0,
                priority_fee_lamports,
                program_config_authority.0,
                multisig_creation_fee,
                treasury.0,
            )
            .await?;
        }
        Command::MultisigCreate {
            keypair,
            config_authority,
            rent_collector,
            members,
            threshold,
            priority_fee_lamports,
        } => {
            let payer = keypair.0;
            let client = Client::new(cluster, payer);
            let program = client.program(program_id)?;
        }
    }
    Ok(())
}
