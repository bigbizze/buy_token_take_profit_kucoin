use anyhow::Result;

use crate::exchange::api_credentials::ApiCredentials;
use crate::exchange::order::{Order, OrderSide};

#[async_trait::async_trait]
pub trait AnExchange {
    async fn new(api_credentials: ApiCredentials) -> Self;
    async fn refresh(&mut self) -> Result<()>;
    async fn limit_order<S, F>(&mut self, symbol: S, quantity: F, price: F, side: OrderSide) -> Result<Order>
        where
            S: Into<String> + Send,
            F: Into<f64> + Send;
    async fn market_order<S, F>(&mut self, symbol: S, quantity: F, side: OrderSide) -> Result<Order>
        where
            S: Into<String> + Send,
            F: Into<f64> + Send;
    async fn cancel_open_orders<S>(&mut self, symbol: S) -> Result<()>
        where S: Into<String> + Send;
}


