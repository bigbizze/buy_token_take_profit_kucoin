use anyhow::Result;

use crate::exchange::api_credentials::ApiCredentials;
use crate::exchange::order::{Order, OrderSide};

#[async_trait::async_trait]
pub trait AnExchange {
    async fn new(api_credentials: ApiCredentials) -> Self;
    async fn refresh(&mut self) -> Result<()>;
    async fn limit_order<S>(&mut self, symbol: S, quantity: S, price: S, side: OrderSide) -> Result<Order>
        where S: Into<String> + Send;
    async fn market_order<S>(&mut self, symbol: S, quantity: S, side: OrderSide) -> Result<Order>
        where S: Into<String> + Send;
    async fn cancel_open_orders<S>(&mut self, symbol: S) -> Result<()>
        where S: Into<String> + Send;
}


