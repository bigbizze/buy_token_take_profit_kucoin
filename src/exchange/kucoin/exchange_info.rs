use std::collections::HashMap;
use std::fmt::Formatter;

use anyhow::{Context, Result};
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv};
use kucoin_rs::kucoin::model::market::SymbolList;

use crate::error::error::MintError;

#[derive(Debug, Clone)]
pub struct KucoinPrecisionInfo {
    pub(crate) quantity_precision: f64,
    pub(crate) price_precision: f64,
}

impl std::fmt::Display for KucoinPrecisionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\"base\":{},\"quote\";{}}}", self.quantity_precision, self.price_precision)
    }
}

pub fn get_one_symbol_info_kc(symbol_info: &SymbolList) -> Result<KucoinPrecisionInfo> {
    let quantity_precision: f64 = symbol_info.base_increment.parse()?;
    let price_precision: f64 = symbol_info.quote_increment.parse()?;
    Ok(KucoinPrecisionInfo {
        quantity_precision,
        price_precision,
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
