use crate::exchange::api_credentials::load_api_credentials;
use crate::user::user::User;
use crate::exchange::kucoin::token_info::SymbolInfo;

pub struct UserManager {
    users: Vec<User>
}

async fn load_users() -> Vec<User> {
    let api_credentials = load_api_credentials();
    api_credentials.into_iter().map(|cred| futures::executor::block_on(User::new(cred))).collect()
}

impl UserManager {
    pub async fn new() -> Self {
        UserManager {
            users: load_users().await
        }
    }
    pub async fn refresh_users(&mut self) {
        for user in &mut self.users {
            user.refresh().await;
        }
    }
    pub async fn buy_tokens(&mut self, symbols: Vec<SymbolInfo>) {
        let mut future_list: Vec<futures::future::BoxFuture<()>> = Vec::new();
        let mut i = 0;
        for user in &mut self.users {
            let buy_promise = user.buy_tokens(symbols.clone());
            future_list.insert(i, Box::pin(buy_promise));
            i += 1;
        }
        futures::future::join_all(future_list).await;
    }
    async fn resolve_place_sell_order(&mut self, symbols: &Vec<SymbolInfo>) -> bool {
        let mut should_continue = true;
        for user in &mut self.users {
            if !user.try_place_sell_limit(symbols).await {
                should_continue = false;
            }
        }
        should_continue
    }
    async fn load_prices(&mut self, symbol_info_vec: Vec<SymbolInfo>) -> Vec<SymbolInfo> {
        let mut new_vec: Vec<SymbolInfo> = Vec::new();
        let mut i = 0;
        for symbol in symbol_info_vec {
            match SymbolInfo::load_price(symbol).await {
                Ok(symbol_with_price) => {
                    new_vec.insert(i, symbol_with_price);
                    i += 1;
                }
                Err(e) => {
                    println!("{}", e)
                }
            }
        }
        new_vec
    }
    pub async fn purchase_and_take_profit(&mut self, symbols: Vec<SymbolInfo>) {
        self.buy_tokens(symbols.clone()).await;
        let symbols = self.load_prices(symbols).await;
        while !self.resolve_place_sell_order(&symbols).await {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}







