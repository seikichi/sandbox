use std::sync::{Arc, Mutex};
use std::{env, error::Error};
use wafwaf::{Params, Response, WafWaf};

fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut wafwaf = WafWaf::new();

    let counter = Arc::new(Mutex::new(0));
    wafwaf.get("/", move |_req, _params: Params| {
        let mut counter = counter.lock().map_err(|err| format!("Error: {}", err))?;
        *counter += 1;
        Ok(Response::builder()
            .header("Content-Type", "text/html")
            .body(format!(
                r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Welcome</title>
  </head>
  <body>
    <p>You are {}th guest.</p>
  </body>
</html>
"#,
                *counter
            ))?)
    });

    wafwaf.listen(&addr)?;

    Ok(())
}
