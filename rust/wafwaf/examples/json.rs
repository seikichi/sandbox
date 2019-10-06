use serde::Serialize;
use std::{env, error::Error};
use wafwaf::{Params, Response, WafWaf};

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut wafwaf = WafWaf::new();

    wafwaf.get("/json", |_req, _params: Params| {
        let body = serde_json::to_string(&Message {
            message: "Hello, World!",
        })?;
        Ok(Response::builder()
            .header("Content-Type", "application/json")
            .body(body)?)
    });

    wafwaf.listen(&addr)?;

    Ok(())
}
