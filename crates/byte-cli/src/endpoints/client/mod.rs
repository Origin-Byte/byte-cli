pub mod create_warehouse;
pub mod deploy_contract;
pub mod mint_nfts;

use std::str::FromStr;

use anyhow::{anyhow, Result};
use package_manager::Network;
use rust_sdk::coin::{select_biggest_coin, select_coin};
use sui_sdk::{
    rpc_types::Coin,
    types::base_types::{ObjectID, SuiAddress},
    wallet_context::WalletContext,
    SuiClient,
};

pub fn get_gas_budget(
    gas_coin: Coin,
    gas_budget: Option<usize>,
) -> Result<usize> {
    let gas_budget = if let Some(gas_budget) = gas_budget {
        if gas_budget as u64 > gas_coin.balance {
            return Err(anyhow!(
                "Gas budget must not be greater than coin balance"
            ));
        }
        gas_budget
    } else {
        (if gas_coin.balance < 50000000000 {
            gas_coin.balance
        } else {
            50000000000
        }) as usize
    };

    Ok(gas_budget)
}

pub async fn get_gas_coin(
    client: &SuiClient,
    sender: SuiAddress,
    gas_coin: Option<String>,
) -> Result<Coin> {
    let gas_coin = if let Some(gas_coin) = gas_coin {
        let gas_coin =
            ObjectID::from_str(gas_coin.as_str()).map_err(|err| {
                anyhow!(r#"Unable to parse gas-id object: {err}"#)
            })?;

        let coin = select_coin(client, sender, gas_coin).await?;
        coin
    } else {
        let coin = select_biggest_coin(client, sender).await?;

        coin
    };

    Ok(gas_coin)
}

pub fn check_network_match(
    wallet_ctx: &WalletContext,
    network: &Network,
) -> Result<()> {
    let network_string = wallet_ctx.config.active_env.as_ref().unwrap();
    let net = Network::from_str(network_string.as_str())
        .map_err(|_| anyhow!("Invalid network string"))?;

    if *network != net {
        return Err(anyhow!(format!(
            "Chosen network {} does not correspond to active Sui network {}",
            network, net
        )));
    }

    Ok(())
}
