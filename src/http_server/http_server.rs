use tide::{Request, Response, StatusCode};


use tide::utils::After;


use std::sync::Arc;
use serde::{Serialize, Deserialize};
use serde_json;
use dotenv::dotenv;
use dotenv_codegen::dotenv;
use anyhow::{ Result };

#[derive(Serialize, Deserialize)]
pub struct ArbitrageRequest {
    tokens: Vec<String>
}

#[derive(Clone)]
struct State {
    pub send_token_s: Arc<tokio::sync::mpsc::Sender<Vec<String>>>
    // pub user_manager: AM<UserManager>
}

const UPSET_SMILEY: &str = ":(";

fn http_ok() -> tide::Result {
    tide::Result::Ok(Response::builder(StatusCode::Ok)
        .build()
    )
}

async fn post_arbitrage(mut req: Request<State>) -> tide::Result {
    let body_string = req.body_string().await?;
    let message = match serde_json::from_str::<ArbitrageRequest>(&*body_string) {
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

pub async fn tide_server(
    send_token_s: tokio::sync::mpsc::Sender<Vec<String>>
    // user_manager: AM<UserManager>
) -> Result<()> {
    /** We are passing a handle to the main thread's tokio futures
        runtime because tide_rs implements futures with async-std instead. */
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

    app.at("/arbitrage").post(post_arbitrage);
    let _ = app.listen(format!("0.0.0.0:{}", port)).await?;
    Ok(())
}
