mod codec;

use codec::Http;

use futures::{SinkExt, StreamExt};
use http::{Request, Response};
use path_tree::PathTree;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::{env, error::Error};
use tokio::codec::Framed;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;

type Params<'a> = Vec<(&'a str, &'a str)>;

trait Handler: Send + Sync + 'static {
    fn handle(&self, r: Request<()>, p: Params) -> Result<Response<String>, Box<dyn Error>>;
}

impl<F> Handler for F
where
    F: Send + Sync + 'static + Fn(Request<()>, Params) -> Result<Response<String>, Box<dyn Error>>,
{
    fn handle(&self, r: Request<()>, p: Params) -> Result<Response<String>, Box<dyn Error>> {
        (*self)(r, p)
    }
}

struct WafWaf {
    tree: PathTree<Box<dyn Handler>>,
}

impl WafWaf {
    fn new() -> Self {
        let tree = PathTree::new();
        Self { tree }
    }

    fn get<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/GET/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    #[allow(dead_code)]
    fn put<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/PUT/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    #[allow(dead_code)]
    fn post<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/POST/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    #[allow(dead_code)]
    fn delete<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/DELETE/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    fn listen(self, addr: &str) -> Result<(), Box<dyn Error>> {
        let rt = Runtime::new().unwrap();
        let tree = Arc::new(self.tree);
        rt.block_on(async {
            let mut incoming = TcpListener::bind(&addr).await?.incoming();

            while let Some(Ok(stream)) = incoming.next().await {
                let tree = tree.clone();
                tokio::spawn(async move {
                    let mut transport = Framed::new(stream, Http);

                    while let Some(request) = transport.next().await {
                        match request {
                            Ok(request) => {
                                let response = {
                                    let path =
                                        format!("/{}/{}", request.method(), request.uri().path());
                                    match tree.find(&path) {
                                        Some((handler, params)) => {
                                            handler.handle(request, params).unwrap()
                                        }
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

    let counter = Arc::new(Mutex::new(0));

    wafwaf.get("/counter", move |_req, _params: Params| {
        let mut counter = counter.lock().unwrap();
        *counter += 1;
        Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .body(format!("Counter: {}\n", *counter))?)
    });

    wafwaf.get("/hello/:name", |_req, params: Params| {
        Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .body(format!("Hello, {}!\n", params[0].1))?)
    });

    wafwaf.get("/json", |_req, _params: Params| {
        #[derive(Serialize)]
        struct Message {
            message: &'static str,
        }
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
