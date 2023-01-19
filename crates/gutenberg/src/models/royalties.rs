use serde::{Deserialize, Serialize};

use crate::GutenError;

#[derive(Debug, Serialize, Deserialize)]
pub enum Royalties {
    Proportional { bps: u64 },
    Constant { fee: u64 },
    None,
}

impl Royalties {
    pub fn new_from(
        input: &str,
        fee: Option<u64>,
    ) -> Result<Royalties, GutenError> {
        match input {
            "Proportional" => {
                let fee = fee.unwrap();
                Ok(Royalties::Proportional { bps: fee })
            }
            "Constant" => {
                let fee = fee.unwrap();
                Ok(Royalties::Constant { fee: fee })
            }
            "None" => Ok(Royalties::None),
            _ => Err(GutenError::UnsupportedRoyalty),
        }
    }

    pub fn write(&self) -> String {
        match self {
            Royalties::Proportional { bps } => {
                format!(
                    "royalty::add_proportional_royalty(
            &mut royalty,
            nft_protocol::royalty_strategy_bps::new({bps}),
        );",
                    bps = bps
                )
            }
            Royalties::Constant { fee } => {
                format!(
                    "royalty::add_constant_royalty(
            &mut royalty,
            nft_protocol::royalty_strategy_bps::new({fee}),
        );",
                    fee = fee
                )
            }
            Royalties::None => "".to_string(),
        }
    }
}
