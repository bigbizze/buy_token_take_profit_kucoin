use anyhow::Result;
use dotenv::dotenv;
use dotenv_codegen::dotenv;

use crate::exchange::an_exchange::AnExchange;
use crate::exchange::api_credentials::ApiCredentials;
use crate::exchange::kucoin::kucoin::KucoinExchange;
use crate::exchange::order::{Order, OrderSide};
use crate::exchange::kucoin::token_info::SymbolInfo;

pub struct User {
    balance: f32,
    exchange: KucoinExchange,
    api_credentials: ApiCredentials,
    alive: bool,
    health: i8,
    active_orders: Vec<Order>,
    take_profit_perc: f32,
    balance_perc: f32,
}

impl User {
    pub async fn new(api_credentials: ApiCredentials) -> Self {
        let mut exchange = KucoinExchange::new(api_credentials.clone()).await;
        let balance = exchange.get_denomination_balance().await.expect(&*format!("Could not get balance for {} on startup!", &api_credentials.name));
        dotenv().ok();
        User {
            balance,
            exchange,
            api_credentials,
            alive: true,
            health: 10,
            active_orders: Vec::new(),
            take_profit_perc: String::from(dotenv!("TAKE_PROFIT_PERC"))
                .parse::<f32>()
                .expect("Got bad value for TAKE_PROFIT_PERC!"),
            balance_perc: String::from(dotenv!("BALANCE_PERC"))
                .parse::<f32>()
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
        Ok(self.balance = self.exchange.get_denomination_balance().await?)
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
    async fn buy_token(&mut self, symbol_info: SymbolInfo, num_tokens: f32) {
        if !self.alive {
            println!("Tried to buy but user {} is dead!", &self.api_credentials.name);
            return;
        }
        let balance_per_token = self.balance * self.balance_perc / num_tokens;
        if let Ok(funds) = self.exchange.round_to_sig_digits_price(&symbol_info.symbol_with_pair, balance_per_token) {
            match self.exchange.market_order(
                symbol_info.symbol_with_pair,
                funds,
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
    }
    pub async fn buy_tokens(&mut self, symbols: Vec<SymbolInfo>) {
        let num_tokens = symbols.len() as f32;
        for symbol in symbols {
            self.buy_token(symbol, num_tokens).await;
        }
    }
    async fn try_place_one_sell_limit(&mut self, symbol_info: &SymbolInfo) -> bool {
        let mut acc_errors: i8 = 0;
        let mut all_orders_finished = true;
        for order in &mut self.active_orders {
            if !order.alive || order.symbol != symbol_info.symbol_with_pair {
                continue;
            }
            let quantity = match self.exchange.get_balance_of(&*symbol_info.symbol.clone()).await {
                Ok(t) => t,
                Err(e) => {
                    println!("{}", e);
                    all_orders_finished = false;
                    order.lower_health();
                    continue;
                }
            };
            let price = symbol_info.price.unwrap();
            let price = price + (price * self.take_profit_perc);
            if let Ok(price) = self.exchange.round_to_sig_digits_price(&symbol_info.symbol_with_pair, price) {
                if let Ok(quantity) = self.exchange.round_to_sig_digits_base(&symbol_info.symbol_with_pair, quantity) {
                    match self.exchange.limit_order(
                        &symbol_info.symbol_with_pair,
                        &quantity,
                        &price,
                        OrderSide::Sell
                    ).await {
                        Ok(_) => {
                            order.alive = false;
                        }
                        Err(e) => {
                            println!("{}", e);
                            all_orders_finished = false;
                            acc_errors += 1;
                        }
                    }
                }
            }
        }
        if acc_errors > 0 {
            self.lower_health(acc_errors);
        }
        all_orders_finished
    }
    pub async fn try_place_sell_limit(&mut self, symbols: &Vec<SymbolInfo>) -> bool {
        let mut all_finished = true;
        for symbol in symbols {
            if !self.try_place_one_sell_limit(symbol).await {
                all_finished = false;
            }
        }
        all_finished || !self.alive
    }
}
