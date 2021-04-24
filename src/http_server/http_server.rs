use std::sync::Arc;

use anyhow::Result;
use dotenv::dotenv;
use dotenv_codegen::dotenv;
use serde::{Deserialize, Serialize};
use serde_json;
use tide::{Request, Response, StatusCode};
use tide::utils::After;

#[derive(Serialize, Deserialize)]
pub struct PurchaseAndTakeProfitReq {
    tokens: Vec<String>
}

#[derive(Clone)]
struct State {
    pub send_token_s: Arc<tokio::sync::mpsc::Sender<Vec<String>>>
}

const UPSET_SMILEY: &str = ":(";

fn http_ok() -> tide::Result {
    tide::Result::Ok(Response::builder(StatusCode::Ok)
        .build()
    )
}

async fn post_purchase_and_take_profit(mut req: Request<State>) -> tide::Result {
    let body_string = req.body_string().await?;
    let message = match serde_json::from_str::<PurchaseAndTakeProfitReq>(&*body_string) {
        Ok(t) => tide::Result::Ok(t),
        Err(e) => {
            println!("{}", e);
            tide::Result::Err(tide::Error::from_str(StatusCode::BadRequest, UPSET_SMILEY))
        }
    }?;
    let state = &mut req.state();
    match state.send_token_s.send(message.tokens).await {
        Ok(_) => http_ok(),
        Err(e) => {
            println!("{}", e);
            tide::Result::Err(tide::Error::from_str(StatusCode::InternalServerError, UPSET_SMILEY))
        }
    }
}

pub async fn tide_server(send_token_s: tokio::sync::mpsc::Sender<Vec<String>>) -> Result<()> {
    let mut app = tide::with_state(State {
        send_token_s: Arc::new(send_token_s)
    });
    app.with(After(|mut res: Response| async move {
        if let Some(err) = res.downcast_error::<async_std::io::Error>() {
            println!("FIRE {}", res.error().unwrap());
            println!("{}", err);
            let msg = format!("Error: {:?}", err);
            res.set_status(StatusCode::NotFound);
            res.set_body(msg);
        }
        Ok(res)
    }));
    dotenv().ok();
    let port = dotenv!("PORT");

    app.at("/create_order").post(post_purchase_and_take_profit);
    let _ = app.listen(format!("0.0.0.0:{}", port)).await?;
    Ok(())
}
