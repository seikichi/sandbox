mod codec;
use codec::Http;

use std::error::Error;
use std::sync::Arc;

use path_tree::PathTree;

use futures::{SinkExt, StreamExt};
use tokio::{
    codec::Framed,
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};

pub use http::{Request, Response};

pub type Params<'a> = Vec<(&'a str, &'a str)>;

pub trait Handler: Send + Sync + 'static {
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

#[derive(Default)]
pub struct WafWaf {
    tree: PathTree<Box<dyn Handler>>,
}

impl WafWaf {
    pub fn new() -> Self {
        let tree = PathTree::new();
        Self { tree }
    }

    pub fn get<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/GET/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    pub fn put<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/PUT/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    pub fn post<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/POST/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    pub fn delete<H: Handler>(&mut self, url: &str, handler: H) {
        let path = format!("/DELETE/{}", url);
        self.tree.insert(&path, Box::new(handler));
    }

    pub fn listen(self, addr: &str) -> Result<(), Box<dyn Error>> {
        let rt = Runtime::new()?;
        let tree = Arc::new(self.tree);
        rt.block_on(async {
            let mut incoming = TcpListener::bind(&addr).await?.incoming();

            while let Some(Ok(stream)) = incoming.next().await {
                let tree = tree.clone();
                tokio::spawn(async move {
                    if let Err(e) = WafWaf::process(stream, tree).await {
                        eprintln!("failed to process connection: error = {}", e);
                    }
                });
            }

            Ok(())
        })
    }

    async fn process(
        stream: TcpStream,
        tree: Arc<PathTree<Box<dyn Handler>>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut transport = Framed::new(stream, Http);

        // TODO (seikichi): use while loop instead of if block here
        if let Some(request) = transport.next().await {
            match request {
                Ok(request) => {
                    let response = WafWaf::respond(request, tree.clone()).await?;
                    transport.send(response).await?;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    async fn respond(
        request: Request<()>,
        tree: Arc<PathTree<Box<dyn Handler>>>,
    ) -> Result<Response<String>, Box<dyn Error>> {
        let path = format!("/{}/{}", request.method(), request.uri().path());
        match tree.find(&path) {
            Some((handler, params)) => handler.handle(request, params),
            None => Response::builder()
                .status(404)
                .body("".into())
                .map_err(|e| e.into()),
        }
    }
}
