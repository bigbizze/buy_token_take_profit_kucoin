use anyhow::Result;
use dotenv::dotenv;
use dotenv_codegen::dotenv;

use crate::exchange::an_exchange::AnExchange;
use crate::exchange::api_credentials::ApiCredentials;
use crate::exchange::kucoin::kucoin::KucoinExchange;
use crate::exchange::order::{Order, OrderSide};

pub struct User {
    alive: bool,
    health: i8,
    api_credentials: ApiCredentials,
    exchange: KucoinExchange,
    balance: f64,
    active_orders: Vec<Order>,
    all_sold: bool,
    take_profit_perc: f64,
    balance_perc: f64,
}

impl User {
    pub async fn new(api_credentials: ApiCredentials) -> Self {
        let mut exchange = KucoinExchange::new(api_credentials.clone()).await;
        let balance = exchange.get_balance().await.expect(&*format!("Could not get balance for {} on startup!", &api_credentials.name));
        dotenv().ok();
        User {
            alive: true,
            health: 10,
            api_credentials,
            exchange,
            balance,
            active_orders: Vec::new(),
            all_sold: false,
            take_profit_perc: String::from(dotenv!("TAKE_PROFIT_PERC"))
                .parse::<f64>()
                .expect("Got bad value for TAKE_PROFIT_PERC!"),
            balance_perc: String::from(dotenv!("TAKE_PROFIT_PERC"))
                .parse::<f64>()
                .expect("Got bad value for BALANCE_PERC!"),
        }
    }
    fn remove_dead(&mut self) {
        self.active_orders = self.active_orders.clone().into_iter().filter(|order| order.alive).collect();
    }
    async fn refresh_exchange_connection(&mut self) {
        self.exchange = KucoinExchange::new(self.api_credentials.clone()).await
    }
    async fn refresh_balance(&mut self) -> Result<()> {
        Ok(self.balance = self.exchange.get_balance().await?)
    }
    pub async fn refresh(&mut self) {
        self.refresh_exchange_connection().await;
        match self.refresh_balance().await {
            Err(e) => {
                println!("{}", e);
                self.lower_health(1);
            }
            _ => {}
        }
        self.remove_dead();
    }
    fn lower_health(&mut self, amount: i8) {
        self.health -= amount;
        if self.health <= 0 {
            self.alive = false;
        }
    }
    pub async fn buy_tokens(&mut self, symbols: Vec<String>) {
        for symbol in symbols {
            self.buy_token(symbol).await;
        }
    }
    async fn buy_token<S>(&mut self, symbol: S)
        where S: Into<String> + Send
    {
        if !self.alive {
            println!("Tried to buy but user {} is dead!", &self.api_credentials.name);
            return;
        }
        match self.exchange.market_order(
            symbol.into(),
            self.balance * self.balance_perc,
            OrderSide::Buy,
        ).await {
            Ok(order) => {
                let num_orders = self.active_orders.len();
                self.active_orders.insert(num_orders, order);
            }
            Err(e) => {
                println!("{}", e);
                self.lower_health(1);
            }
        }
    }
    pub async fn try_place_sell_limit(&mut self, symbols: &Vec<String>) -> bool {
        for symbol in symbols {
            self.try_place_one_sell_limit(symbol).await;
        }
        self.all_sold || !self.alive
    }
    async fn try_place_one_sell_limit<S>(&mut self, symbol: S)
        where S: Into<String> + Send
    {
        let symbol = symbol.into();
        let mut acc_errors: i8 = 0;
        for order in &mut self.active_orders {
            let (price, quantity) = match self.exchange.get_price_and_quantity_of_order(&order.order_id).await {
                Ok(t) => t,
                Err(e) => {
                    println!("{}", e);
                    order.lower_health();
                    continue;
                }
            };
            match self.exchange.limit_order(&symbol, quantity, price + (price * self.take_profit_perc), OrderSide::Sell).await {
                Ok(_) => {
                    order.alive = false;
                }
                Err(e) => {
                    println!("{}", e);
                    acc_errors += 1;
                }
            }
        }
        if acc_errors > 0 {
            self.lower_health(acc_errors);
        } else {
            self.all_sold = true;
        }
    }
}
