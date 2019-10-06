use std::{env, error::Error};
use wafwaf::{Params, Response, WafWaf};

fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut wafwaf = WafWaf::new();

    wafwaf.get("/hello", |_req, _params: Params| {
        Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .body(format!("Hello, world!\n"))?)
    });

    wafwaf.get("/hello/:name", |_req, params: Params| {
        Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .body(format!("Hello, {}!\n", params[0].1))?)
    });

    wafwaf.listen(&addr)?;

    Ok(())
}
