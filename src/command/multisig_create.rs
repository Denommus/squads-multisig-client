use std::{error::Error, str::FromStr, sync::Arc};

use anchor_client::{Client, Cluster};
use clap::Args;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_program,
};
use squads_multisig::{
    client::MultisigCreateArgsV2,
    pda::{get_multisig_pda, get_program_config_pda},
    squads_multisig_program::state::ProgramConfig,
    state::{Member, Permissions},
};

use crate::newtypes::{ClapAddress, ClapKeypair};

#[derive(Args)]
pub struct MultisigCreate {
    /// Path to the private key responsible for signing the multisig creation
    #[arg(long)]
    keypair: ClapKeypair,
    /// OPTIONAL, Pubkey of the key responsible for updating the config of the multisig
    #[arg(long)]
    config_authority: Option<ClapAddress>,
    /// "OPTIONAL, Pubkey of the rent collector"
    #[arg(long)]
    rent_collector: Option<ClapAddress>,
    /// List of members. Each member needs to have a pubkey and a permission mask, separated by comma (e.g. <pubkey>,<mask>)
    #[arg(long, short, value_delimiter = ' ')]
    members: Vec<String>,
    #[arg(long)]
    threshold: u16,
    /// OPTIONAL, a private key to make the multisig address deterministic
    #[arg(long)]
    multisig_keypair: Option<ClapKeypair>,
    /// OPTIONAL, the lamports used to increase or decrease the priority of the transaction
    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl MultisigCreate {
    pub async fn execute(self, cluster: Cluster, program_id: Pubkey) -> Result<(), Box<dyn Error>> {
        let MultisigCreate {
            keypair,
            config_authority,
            rent_collector,
            members,
            threshold,
            multisig_keypair,
            priority_fee_lamports,
        } = self;

        let client = Client::new(cluster, keypair.0.clone());

        let program = client.program(program_id)?;

        let program_config_pda = get_program_config_pda(Some(&program_id)).0;

        let multisig_keypair = multisig_keypair
            .map(|x| x.0)
            .unwrap_or_else(|| Arc::new(Keypair::new()));

        let multisig = get_multisig_pda(&multisig_keypair.pubkey(), Some(&program_id)).0;

        let compute_budget_ix =
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports.unwrap_or(5000));

        let program_config: ProgramConfig = program.account(program_config_pda).await?;

        let treasury = program_config.treasury;

        let ix = squads_multisig::client::multisig_create_v2(
            squads_multisig::squads_multisig_program::accounts::MultisigCreateV2 {
                program_config: program_config_pda,
                treasury,
                multisig,
                create_key: multisig_keypair.pubkey(),
                creator: keypair.0.pubkey(),
                system_program: system_program::ID,
            },
            MultisigCreateArgsV2 {
                config_authority: config_authority.map(|x| x.0),
                threshold,
                members: parse_members(members)?,
                memo: None,
                time_lock: 0,
                rent_collector: rent_collector.map(|x| x.0),
            },
            Some(program_id),
        );

        let signature = program
            .request()
            .instruction(compute_budget_ix)
            .instruction(ix)
            .signer(&keypair.0)
            .signer(&multisig_keypair)
            .send()
            .await?;

        println!("Multisig: {multisig}");
        println!("Signature: {signature}");

        Ok(())
    }
}

/// Taken from https://github.com/Squads-Protocol/v4
/// Copyright Squads Protocol
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
