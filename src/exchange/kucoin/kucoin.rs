use std::collections::HashMap;

use anyhow::{Context, Result};
use kucoin_rs_custom::kucoin::client::{Credentials, Kucoin, KucoinEnv};
use kucoin_rs_custom::kucoin::model::APIDatum;
use kucoin_rs_custom::kucoin::model::user::AccountType;

use crate::error::error::MintError;
use crate::exchange::an_exchange::AnExchange;
use crate::exchange::api_credentials::ApiCredentials;
use crate::exchange::kucoin::exchange_info::{get_exchange_info_kc, KucoinPrecisionInfo};
use crate::exchange::order::*;
use crate::utils::time::get_ms_str;

pub struct KucoinExchange {
    pub account: Kucoin,
    pub exchange_info: HashMap<String, KucoinPrecisionInfo>
}

pub const DENOMINATION: &'static str = "BTC";

impl KucoinExchange {
    pub async fn get_balance_of(&mut self, symbol: &str) -> Result<f32> {
        let bal = self.account.get_transferable_balance(symbol, AccountType::Trade)
            .await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to get Kucoin transferable balance!"))?;
        Ok(KucoinExchange::unwrap_data(bal)?.balance.parse::<f32>()?)
    }
    pub async fn get_denomination_balance(&mut self) -> Result<f32> {
        self.get_balance_of(DENOMINATION).await
    }
    pub fn unwrap_data<T>(res: APIDatum<T>) -> Result<T> {
        let msg = &res.msg;
        Ok(res.data.ok_or_else(|| {
            if let Some(msg) = msg {
                MintError::from_str(format!("Error in unwrap_data!\n{}", msg.clone()))
            } else {
                MintError::from_str(format!("Didn't get any data in unwrap_data!"))
            }
        })?)
    }
    pub fn round_to_sig(price: f32, sig_digits: i8) -> Result<String> {
        let res_chars: Vec<char> = price.to_string().chars().collect();
        let mut i: usize = 0;
        let dec_char = '.';
        for char in &res_chars {
            i += 1;
            if char == &dec_char {
                break;
            }
        }
        let sig_digits = sig_digits as usize;
        if res_chars.len() - i <= sig_digits {
            return Ok(price.to_string());
        }
        let new_arr: &[char] = &res_chars[0..sig_digits + i];
        let val: String = new_arr.iter().collect();
        Ok(val)
    }
    pub fn round_to_sig_digits_price(&mut self, symbol: &String, val: f32) -> Result<String> {
        let price_sig_digits = self.exchange_info
            .get_mut(symbol)
            .ok_or_else(|| MintError::from_str(format!("Could not get info for symbol {}", symbol)))?
            .price_sig_digs;
        Ok(KucoinExchange::round_to_sig(val, price_sig_digits)?)
    }
    pub fn round_to_sig_digits_base(&mut self, symbol: &String, val: f32) -> Result<String> {
        let quantity_sig_digits = self.exchange_info
            .get_mut(symbol)
            .ok_or_else(|| MintError::from_str(format!("Could not get info for symbol {}", symbol)))?
            .base_sig_digs;
        Ok(KucoinExchange::round_to_sig(val, quantity_sig_digits)?)
    }
}

#[async_trait::async_trait]
impl AnExchange for KucoinExchange {
    async fn new(api_credentials: ApiCredentials) -> Self {
        let exchange_info = match get_exchange_info_kc().await {
            Ok(t) => t,
            Err(e) => panic!("{}", e)
        };
        let account = match Kucoin::new(KucoinEnv::Live, Some(Credentials::new(&api_credentials.api_key, &api_credentials.api_secret, &api_credentials.api_pass))) {
            Ok(t) => t,
            Err(e) => panic!("{}", MintError::from_kucoin_err(e.into()).get_fmt_error())
        };
        KucoinExchange {
            account,
            exchange_info
        }
    }

    async fn refresh(&mut self) -> Result<()> {
        self.exchange_info = get_exchange_info_kc().await?;
        Ok(())
    }

    async fn limit_order<S>(&mut self, symbol: S, quantity: S, price: S, side: OrderSide) -> Result<Order>
        where S: Into<String> + Send
    {
        let kind = OrderKind::Limit;
        let side_text = match &side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell"
        };
        let symbol = symbol.into();
        let quantity = quantity.into();
        let price = price.into();
        let order_res = self.account.post_limit_order(
            &get_ms_str()?,
            &symbol,
            side_text,
            &*price,
            &*quantity,
            None,
        ).await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to create Kucoin limit {} order!", &side))?;
        let order_id = KucoinExchange::unwrap_data(order_res)?.order_id;
        Ok(Order {
            symbol: symbol.clone(),
            order_id: String::from(order_id),
            kind: Some(kind),
            side: Some(side),
            health: 5,
            alive: true
        })
    }

    async fn market_order<S>(&mut self, symbol: S, funds: S, side: OrderSide) -> Result<Order>
        where S: Into<String> + Send
    {
        let kind = OrderKind::Market;
        let side_text = match &side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell"
        };
        let symbol = symbol.into();
        let funds = funds.into();
        let order_res = self.account.post_market_order(
            &get_ms_str()?,
            &symbol,
            side_text,
            None,
            Some(funds),
            None,
        ).await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to create Kucoin limit {} order!", &side))?;
        let order_id = KucoinExchange::unwrap_data(order_res)?.order_id;
        Ok(Order {
            symbol,
            order_id: String::from(order_id),
            kind: Some(kind),
            side: Some(side),
            health: 5,
            alive: true
        })
    }

    async fn cancel_open_orders<S>(&mut self, symbol: S) -> Result<()> where S: Into<String> + Send {
        self.account.cancel_all_orders(Some(&symbol.into()), Some("TRADE"))
            .await.map_err(|e| MintError::from_kucoin_err(e.into()))?;
        Ok(())
    }
}




