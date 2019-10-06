mod codec;

use codec::Http;

use futures::{SinkExt, StreamExt};
use http::{Request, Response};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, error::Error};
use tokio::codec::Framed;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

type Callback = Box<dyn Fn(Request<()>) -> Response<String> + Send + Sync>;

struct WafWaf {
    routes: HashMap<String, Callback>,
}

impl WafWaf {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn get<C: Fn(Request<()>) -> Response<String> + 'static + Send + Sync>(
        &mut self,
        url: &str,
        callback: C,
    ) {
        self.routes.insert(url.to_string(), Box::new(callback));
    }

    fn listen(self, addr: &str) -> Result<(), Box<dyn Error>> {
        let rt = Runtime::new().unwrap();
        let routes = Arc::new(self.routes);
        rt.block_on(async {
            let mut incoming = TcpListener::bind(&addr).await?.incoming();

            while let Some(Ok(stream)) = incoming.next().await {
                let routes = routes.clone();
                tokio::spawn(async move {
                    let mut transport = Framed::new(stream, Http);

                    while let Some(request) = transport.next().await {
                        match request {
                            Ok(request) => {
                                let response = {
                                    match routes.get(request.uri().path()) {
                                        Some(callback) => callback(request),
                                        _ => panic!("HAHAHA"),
                                    }
                                };
                                transport.send(response).await.unwrap();
                            }
                            Err(e) => {
                                println!("failed to process connection; error = {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Ok(())
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let mut wafwaf = WafWaf::new();

    wafwaf.get("/hello", |_req| {
        let mut response = Response::builder();
        response.header("Content-Type", "text/plain");
        response.body("Hello, World!".to_string()).unwrap()
    });

    wafwaf.get("/json", |_req| {
        let mut response = Response::builder();
        response.header("Content-Type", "application/json");

        #[derive(Serialize)]
        struct Message {
            message: &'static str,
        }
        let body = serde_json::to_string(&Message {
            message: "Hello, World!",
        })
        .unwrap();
        response.body(body).unwrap()
    });

    wafwaf.listen(&addr)?;

    Ok(())
}
