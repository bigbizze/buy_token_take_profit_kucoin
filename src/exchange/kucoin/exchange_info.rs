use std::collections::HashMap;

use anyhow::{Context, Result};
use kucoin_rs_custom::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs_custom::kucoin::model::market::SymbolList;
use serde::{Serialize, Deserialize};
use crate::error::error::MintError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KucoinPrecisionInfo {
    pub(crate) base_sig_digs: i8,
    pub(crate) price_sig_digs: i8
}

pub fn num_sig_digits(float_str: String) -> i8 {
    let char_vec: Vec<char> = float_str.chars().collect();
    let mut is_decimal_seen = false;
    let mut num_sig_digits: i8 = 0;
    for c in char_vec {
        if c == '.' {
            is_decimal_seen = true;
        } else if is_decimal_seen {
            num_sig_digits += 1;
        }
    }
    num_sig_digits
}

pub fn get_one_symbol_info_kc(symbol_info: &SymbolList) -> Result<KucoinPrecisionInfo> {
    let base_sig_digs: i8 = num_sig_digits(symbol_info.base_increment.clone());
    let price_sig_digs: i8 = num_sig_digits(symbol_info.price_increment.clone());
    Ok(KucoinPrecisionInfo {
        price_sig_digs,
        base_sig_digs
    })
}

pub async fn get_exchange_info_kc() -> Result<HashMap<String, KucoinPrecisionInfo>> {
    let client = get_anon_kc_client()?;
    let mut exchange_info_map: HashMap<String, KucoinPrecisionInfo> = HashMap::new();
    let exchange_info = client.get_symbol_list(None).await
        .map_err(|e| MintError::from_kucoin_err(e.into()))
        .with_context(|| MintError::from_str(format!("Error getting symbol list from Kucoin.")))?
        .data.ok_or_else(|| MintError::from_str(format!("Didn't get any data in get_exchange_info_kc!")))?;
    for symbol in exchange_info.iter() {
        let one_symbol = match get_one_symbol_info_kc(symbol) {
            Ok(t) => t,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        exchange_info_map.insert(
            symbol.symbol.to_string(),
            one_symbol,
        );
    }
    Ok(exchange_info_map)
}

pub fn get_anon_kc_client() -> Result<Kucoin> {
    let client = Kucoin::new(KucoinEnv::Live, None)
        .map_err(|e| MintError::from_kucoin_err(e.into()))
        .with_context(|| format!("Failed to get anonymous Kucoin client!"))?;
    Ok(client)
}
