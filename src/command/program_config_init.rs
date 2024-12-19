use std::error::Error;

use anchor_client::{
    anchor_lang::{InstructionData, ToAccountMetas},
    Client, Cluster,
};
use clap::Args;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, pubkey::Pubkey,
    signer::Signer, system_program,
};
use squads_multisig::{
    pda::get_program_config_pda, squads_multisig_program::ProgramConfigInitArgs,
};

use crate::newtypes::{ClapAddress, ClapKeypair};

#[derive(Args)]
pub struct ProgramConfigInit {
    /// Path to the private key responsible for signing the init transaction
    #[arg(long)]
    pub initializer_keypair: ClapKeypair,
    /// Pubkey of the key responsible for updating the config
    #[arg(long)]
    pub program_config_authority: ClapAddress,
    /// Pubkey of the treasury
    #[arg(long)]
    pub treasury: ClapAddress,
    /// Fee for creating a multisig account
    #[arg(long)]
    pub multisig_creation_fee: u64,
    /// OPTIONAL, the lamports used to increase or decrease the priority of the transaction
    #[arg(long)]
    pub priority_fee_lamports: Option<u64>,
}

impl ProgramConfigInit {
    pub async fn execute(self, cluster: Cluster, program_id: Pubkey) -> Result<(), Box<dyn Error>> {
        let ProgramConfigInit {
            initializer_keypair,
            program_config_authority,
            treasury,
            multisig_creation_fee,
            priority_fee_lamports,
        } = self;
        let client = Client::new(cluster, initializer_keypair.0.clone());
        let program = client.program(program_id)?;
        let program_config = get_program_config_pda(Some(&program_id)).0;

        let compute_budget_ix =
            ComputeBudgetInstruction::set_compute_unit_price(priority_fee_lamports.unwrap_or(5000));
        let ix = Instruction {
            accounts: squads_multisig::squads_multisig_program::accounts::ProgramConfigInit {
                program_config,
                initializer: initializer_keypair.0.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(Some(false)),
            data: squads_multisig::squads_multisig_program::instruction::ProgramConfigInit {
                args: ProgramConfigInitArgs {
                    authority: program_config_authority.0,
                    multisig_creation_fee,
                    treasury: treasury.0,
                },
            }
            .data(),
            program_id,
        };

        let signature = program
            .request()
            .instruction(compute_budget_ix)
            .instruction(ix)
            .signer(&initializer_keypair.0)
            .send()
            .await?;

        println!("Program config: {}", program_config);
        println!("Transaction signature: {}", signature);

        Ok(())
    }
}
