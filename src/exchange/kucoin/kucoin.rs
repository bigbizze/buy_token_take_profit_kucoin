use anyhow::{Result, Context};
use std::collections::HashMap;
use kucoin_rs::kucoin::client::{Kucoin, KucoinEnv, Credentials};
use crate::exchange::kucoin::exchange_info::{KucoinPrecisionInfo, get_exchange_info_kc};
use crate::error::error::MintError;
use crate::utils::time::{get_ms_str};
use kucoin_rs::kucoin::model::user::AccountType;
use kucoin_rs::kucoin::model::trade::OrderInfo;
use kucoin_rs::kucoin::model::APIDatum;
use crate::exchange::an_exchange::AnExchange;
use crate::exchange::api_credentials::ApiCredentials;
use crate::exchange::order::*;

pub struct KucoinExchange {
    pub account: Kucoin,
    pub exchange_info: HashMap<String, KucoinPrecisionInfo>
}

const DENOMINATION: &'static str = "USDT";

impl KucoinExchange {
    pub async fn get_balance(&mut self) -> Result<f64> {
        let bal_res = self.account.get_transferable_balance(&DENOMINATION, AccountType::Trade)
            .await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to get Kucoin transferable balance!"))?;
        Ok(KucoinExchange::unwrap_data(bal_res)?.balance.parse::<f64>()?)
    }

    pub async fn get_price_and_quantity_of_order(&mut self, order_id: &String) -> Result<(f64, f64)> {
        let order_res = self.account.get_order(order_id)
            .await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to get Kucoin transferable balance after placing order!"))?;
        match KucoinExchange::unwrap_data(order_res)? {
            OrderInfo { price, size, .. } => Ok((price.parse::<f64>()?, size.parse::<f64>()?))
        }
    }

    pub fn unwrap_data<T>(res: APIDatum<T>) -> Result<T> {
        Ok(res.data.ok_or_else(|| MintError::from_str(format!("Didn't get any data in get_exchange_info_kc!")))?)
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

    async fn limit_order<S, F>(&mut self, symbol: S, quantity: F, price: F, side: OrderSide) -> Result<Order>
        where
            S: Into<String> + Send,
            F: Into<f64> + Send
    {
        let kind = OrderKind::Limit;
        let side_text = match &side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell"
        };
        let symbol = symbol.into();
        let order_res = self.account.post_limit_order(
            &get_ms_str()?,
            &symbol,
            side_text,
            &*price.into().to_string(),
            &*quantity.into().to_string(),
            None
        ).await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to create Kucoin limit {} order!", &side))?;
        let order_id = KucoinExchange::unwrap_data(order_res)?.order_id;
        Ok(Order {
            order_id: String::from(order_id),
            kind: Some(kind),
            side: Some(side),
            health: 0,
            alive: true
        })
    }

    async fn market_order<S, F>(&mut self, symbol: S, quantity: F, side: OrderSide) -> Result<Order>
        where
            S: Into<String> + Send,
            F: Into<f64> + Send
    {
        let kind = OrderKind::Market;
        let side_text = match &side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell"
        };
        let symbol = symbol.into();
        let order_res = self.account.post_market_order(
            &get_ms_str()?,
            &symbol,
            side_text,
            None,
            Some(quantity.into() as f32),
            None
        ).await.map_err(|e| MintError::from_kucoin_err(e.into()))
            .context(format!("Failed to create Kucoin limit {} order!", &side))?;
        let order_id = KucoinExchange::unwrap_data(order_res)?.order_id;
        Ok(Order {
            order_id: String::from(order_id),
            kind: Some(kind),
            side: Some(side),
            health: 0,
            alive: true
        })
    }

    async fn cancel_open_orders<S>(&mut self, symbol: S) -> Result<()> where S: Into<String> + Send {
        self.account.cancel_all_orders(Some(&symbol.into()), Some("TRADE"))
            .await.map_err(|e| MintError::from_kucoin_err(e.into()))?;
        Ok(())
    }
}




