extern crate tela;

use hyper::{body::Incoming, Response};
use serde::Deserialize;
use std::sync::Arc;
use tela::{
    client::{fetch, SendRequest},
    prelude::*,
    response::{html, json, HTML},
    server::{
        router::{get, post},
        Router, Server, Socket,
    },
    Request,
};

#[derive(Deserialize, Debug)]
struct Query {
    firstname: String,
    lastname: String,
}

#[derive(Deserialize, Debug)]
struct Body {
    r#type: String,
    message: String,
    length: u32,
}

async fn posted(req: Request) -> impl IntoResponse {
    // Any method parsing into an object returns a result
    let query: Query = req.query().unwrap();
    let body: Body = req.json().await.unwrap();

    html::new! {
        <ul>
            <li><strong>"Type: "</strong>  {body.r#type}</li>
            <li><strong>"First: "</strong>  {query.firstname}</li>
            <li><strong>"Last: "</strong>   {query.lastname}</li>
            <li><strong>"Message: "</strong>{body.message}</li>
            <li><strong>"Length: "</strong> {body.length}</li>
        </ul>
    }
}

#[tokio::main]
async fn main() {
    const url: &'static str = "http://127.0.0.1:3000/posted?firstname=Tela&lastname=Web";

    Server::builder()
        .on_bind(|addr| println!("Serving to {}", addr))
        .serve(
            Socket::Local(3000),
            Router::new()
                .route("/posted", post(posted))
                .route(
                    "/macro",
                    get(|_| async {
                        let response: Response<Incoming> = fetch!(
                            url,
                            method: post,
                            body: json::new!({
                                "type": "macro",
                                "message": "Hello, world!",
                                "length": 13
                            })
                        )
                        .await;

                        match response.text().await {
                            Ok(text) => HTML(text),
                            Err(e) => html::from!(<strong>"Error: "{e}</strong>),
                        }
                    }),
                )
                .route(
                    "/raw",
                    get(|_| async {
                        let response = Request::builder()
                            .uri(url)
                            .method("POST")
                            .body(json::new!({
                                "type": "raw",
                                "message": "Hello, world!",
                                "length": 13
                            }))
                            .send()
                            .await;

                        match response.text().await {
                            Ok(text) => HTML(text),
                            Err(e) => html::from!(<strong>"Error: "{e}</strong>),
                        }
                    }),
                ),
        )
        .await
}
