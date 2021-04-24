use crate::exchange::api_credentials::load_api_credentials;
use crate::user::user::User;

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
    pub async fn buy_tokens(&mut self, symbols: Vec<String>) {
        let mut future_list: Vec<futures::future::BoxFuture<()>> = Vec::new();
        let mut i = 0;
        for user in &mut self.users {
            let buy_promise = user.buy_tokens(symbols.clone());
            future_list.insert(i, Box::pin(buy_promise));
            i += 1;
        }
        futures::future::join_all(future_list).await;
    }
    async fn resolve_place_sell_order(&mut self, symbols: &Vec<String>) -> bool {
        let mut should_continue = true;
        for user in &mut self.users {
            if !user.try_place_sell_limit(symbols).await {
                should_continue = false;
            }
        }
        should_continue
    }
    pub async fn execute_arbitrage(&mut self, symbols: Vec<String>) {
        self.buy_tokens(symbols.clone()).await;
        while !self.resolve_place_sell_order(&symbols).await {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
}







