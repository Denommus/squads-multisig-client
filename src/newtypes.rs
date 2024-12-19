use std::{str::FromStr, sync::Arc};

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::EncodableKey};

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
