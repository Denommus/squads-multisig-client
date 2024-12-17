use std::{error::Error, str::FromStr, sync::Arc};

use anchor_client::{
    anchor_lang::{system_program, InstructionData, ToAccountMetas},
    Client, Cluster, Program,
};
use clap::{Parser, Subcommand};
use solana_program::pubkey;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};
use squads_multisig::{
    client::MultisigCreateArgsV2,
    pda::{get_multisig_pda, get_program_config_pda},
    squads_multisig_program::ProgramConfigInitArgs,
    state::{Member, Permissions},
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
        multisig_pubkey: Option<ClapAddress>,
        #[arg(long)]
        treasury: ClapAddress,
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
    program: &Program<Arc<Keypair>>,
    program_id: Pubkey,
    payer: &Keypair,
    priority_fee_lamports: Option<u64>,
    program_config_authority: Pubkey,
    multisig_creation_fee: u64,
    treasury: Pubkey,
) -> Result<(), Box<dyn Error>> {
    let program_config = get_program_config_pda(Some(&program_id)).0;

    let compute_budget_ix =
        ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports.unwrap_or(5000));
    let ix = Instruction {
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
    };

    let signature = program
        .request()
        .instruction(compute_budget_ix)
        .instruction(ix)
        .signer(payer)
        .send()
        .await?;

    println!("Program config: {}", program_config);
    println!("Transaction signature: {}", signature);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn multisig_create(
    program: &Program<Arc<Keypair>>,
    program_id: Pubkey,
    treasury: Pubkey,
    multisig_pubkey: Option<Pubkey>,
    keypair: &Keypair,
    config_authority: Option<Pubkey>,
    rent_collector: Option<Pubkey>,
    threshold: u16,
    members: Vec<Member>,
    priority_fee_lamports: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    let program_config = get_program_config_pda(Some(&program_id)).0;

    let multisig_pubkey = multisig_pubkey.unwrap_or_else(|| {
        let keypair = Keypair::new();
        keypair.pubkey()
    });

    let multisig = get_multisig_pda(&multisig_pubkey, Some(&program_id)).0;

    let compute_budget_ix =
        ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports.unwrap_or(5000));

    let ix = squads_multisig::client::multisig_create_v2(
        squads_multisig::squads_multisig_program::accounts::MultisigCreateV2 {
            program_config,
            treasury,
            multisig,
            create_key: multisig_pubkey,
            creator: keypair.pubkey(),
            system_program: system_program::ID,
        },
        MultisigCreateArgsV2 {
            config_authority,
            threshold,
            members,
            memo: None,
            time_lock: 0,
            rent_collector,
        },
        Some(program_id),
    );

    let signature = program
        .request()
        .instruction(compute_budget_ix)
        .instruction(ix)
        .signer(keypair)
        .send()
        .await?;

    println!("Multisig: {multisig}");
    println!("Signature: {signature}");

    Ok(())
}

fn parse_members(member_strings: Vec<String>) -> Result<Vec<Member>, String> {
    member_strings
        .into_iter()
        .map(|s| {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() != 2 {
                return Err(
                    "Each entry must be in the format <public_key>,<permission>".to_string()
                );
            }

            let key =
                Pubkey::from_str(parts[0]).map_err(|_| "Invalid public key format".to_string())?;
            let permissions = parts[1]
                .parse::<u8>()
                .map_err(|_| "Invalid permission format".to_string())?;

            Ok(Member {
                key,
                permissions: Permissions { mask: permissions },
            })
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = App::parse();

    let cluster = match app.rpc_url {
        Some(address) => Cluster::Custom(address.clone(), address),
        None => Cluster::Localnet,
    };

    let program_id: Pubkey = app
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
            let client = Client::new(cluster, initializer_keypair.0.clone());
            let program = client.program(program_id)?;
            program_config_init(
                &program,
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
            multisig_pubkey,
            treasury,
            priority_fee_lamports,
        } => {
            let payer = keypair.0;
            let client = Client::new(cluster, payer.clone());
            let program = client.program(program_id)?;
            let config_authority = config_authority.map(|x| x.0);
            let rent_collector = rent_collector.map(|x| x.0);
            let multisig_pubkey = multisig_pubkey.map(|x| x.0);
            let members = parse_members(members)?;
            multisig_create(
                &program,
                program_id,
                treasury.0,
                multisig_pubkey,
                &payer,
                config_authority,
                rent_collector,
                threshold,
                members,
                priority_fee_lamports,
            )
            .await?;
        }
    }
    Ok(())
}
