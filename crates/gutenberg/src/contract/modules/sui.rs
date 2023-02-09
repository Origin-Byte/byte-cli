use serde::{Deserialize, Serialize};

use super::Module;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Url();

impl Module for Url {
    fn import(&self) -> String {
        "use sui::url;".to_string()
    }
}

impl Url {
    pub fn to_url_param(url: &str, is_var_name: bool) -> String {
        if is_var_name {
            // e.g. url = "url"
            format!("sui::url::new_unsafe_from_bytes({}),", url)
        } else {
            // e.g. url = "www.originbyte.io"
            format!("sui::url::new_unsafe_from_bytes(b\"{}\"),", url)
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance();

impl Module for Balance {
    fn import(&self) -> String {
        "use sui::balance;".to_string()
    }
}

impl Balance {
    pub fn balance_mut_expr(var: &str) -> String {
        format!("royalties::balance_mut(Witness {{}}, {var});", var = var)
    }

    pub fn balance_value(var: &str) -> String {
        format!("balance::value({})", var)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer();

impl Module for Transfer {
    fn import(&self) -> String {
        "use sui::transfer;".to_string()
    }
}

impl Transfer {
    pub fn tranfer_to_sender(obj: &str) -> String {
        format!("transfer::transfer({}, tx_context::sender(ctx));", obj)
    }

    pub fn share(obj: &str) -> String {
        format!("transfer::share_object({});", obj)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxContext();

impl Module for TxContext {
    fn import(&self) -> String {
        "use sui::tx_context::{Self, TxContext};".to_string()
    }
}

impl TxContext {
    pub fn sender(_obj: &str) -> String {
        "tx_context::sender(ctx)".to_string()
    }
}
