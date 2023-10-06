extern crate tela;

use hyper::{body::Incoming, Response};
use serde::Deserialize;
use tela::{
    client::{fetch, SendRequest},
    prelude::*,
    response::HTML,
    server::{
        router::{get, post},
        serve, Router, Socket,
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
    message: String,
    length: u32,
}

async fn posted(req: Request) -> HTML<String> {
    // Any method parsing into an object returns a result
    let query: Query = req.query().unwrap();
    let body: Body = req.json().await.unwrap();

    html! {
        <ul>
            <li><strong>"First: "</strong>  {query.firstname}</li>
            <li><strong>"Last: "</strong>   {query.lastname}</li>
            <li><strong>"Message: "</strong>{body.message}</li>
            <li><strong>"Length: "</strong> {body.length}</li>
        </ul>
    }
}

#[tokio::main]
async fn main() {
    let url = "http://127.0.0.1:3000/posted?firstname=Tela&lastname=Web".to_string();

    serve(
        Socket::Local(3000),
        Router::new()
            .route("/posted", post(posted))
            .route(
                "/raw",
                get(|_| async {
                    let response = Request::builder()
                        .uri("http://127.0.0.1:3000/posted?firstname=Tela&lastname=Web")
                        .method("POST")
                        .body(json!({
                            "message": "Hello, world!",
                            "length": 13
                        }))
                        .send()
                        .await;

                    match response.text().await {
                        Ok(text) => HTML(text),
                        Err(e) => html!(<strong>"Error: "{e}</strong>),
                    }
                }),
            )
            .route(
                "/macro",
                get(|_| async {
                    let response: Response<Incoming> = fetch!(
                        "http://127.0.0.1:3000/posted?firstname=Tela&lastname=Web",
                        method: post,
                        body: json!({
                            "message": "Hello, world!",
                            "length": 13
                        })
                    )
                    .await;

                    match response.text().await {
                        Ok(text) => HTML(text),
                        Err(e) => html!(<strong>"Error: "{e}</strong>),
                    }
                }),
            ),
    )
    .await
}
