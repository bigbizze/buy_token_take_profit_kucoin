use std::sync::Arc;

use tokio::sync::Mutex;

use crate::http_server::http_server::tide_server;
use crate::user::user_manager::UserManager;
use exchange::kucoin::token_info::make_token_info_vec;

mod http_server;
mod exchange;
mod user;
mod utils;
mod error;


#[tokio::main]
async fn main() {
    let user_manager = Arc::new(Mutex::new(UserManager::new().await));
    let user_manager_ref1 = Arc::clone(&user_manager);
    let user_manager_ref2 = Arc::clone(&user_manager);
    let (send_token_s, mut token) = tokio::sync::mpsc::channel::<Vec<String>>(24);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
            {
                let mut um = user_manager_ref1.lock().await;
                um.refresh_users().await;
            }
        }
    });
    tokio::spawn(async move {
        loop {
            let maybe_tokens = token.recv().await.ok_or_else(|| println!("Failed attempting to receive tokens from server!")).ok();
            if let Some(tokens) = maybe_tokens {
                let tokens = make_token_info_vec(tokens);
                let mut um = user_manager_ref2.lock().await;
                um.purchase_and_take_profit(tokens).await;
            }
        }
    });
    tide_server(send_token_s).await.expect("tide server failed!");
}
