use anyhow::{Context, Result};
use crate::exchange::kucoin::exchange_info::get_anon_kc_client;
use crate::error::error::MintError;
use crate::exchange::kucoin::kucoin::KucoinExchange;

#[derive(Clone)]
pub struct SymbolInfo {
    pub symbol: String,
    pub symbol_with_pair: String,
    pub price: Option<f32>
}

impl SymbolInfo {
    pub async fn load_price(symbol_info: SymbolInfo) -> Result<SymbolInfo> {
        let kc = get_anon_kc_client()?;
        let ticker = kc.get_ticker(&symbol_info.symbol_with_pair).await
            .map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Could not get symbol price for {}!", &symbol_info.symbol))?;
        Ok(SymbolInfo {
            symbol: symbol_info.symbol,
            symbol_with_pair: symbol_info.symbol_with_pair,
            price: Some(KucoinExchange::unwrap_data(ticker)?.price.parse::<f32>()?)
        })
    }
}

const DENOMINATION: &'static str = "BTC";

pub fn prep_symbol_for_kucoin(symbol: String) -> String {
    format!("{}-{}", symbol.to_uppercase(), DENOMINATION)
}

pub fn make_token_info_vec(symbols: Vec<String>) -> Vec<SymbolInfo> {
    symbols.into_iter().map(|symbol| {
        let symbol = symbol.to_uppercase();
        let symbol_with_pair = prep_symbol_for_kucoin(symbol.clone());
        SymbolInfo {
            symbol,
            symbol_with_pair,
            price: None
        }
    }).collect()
}
