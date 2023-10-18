extern crate tela;

use tela::{
    cookie::{Cookie, Cookies},
    html::{self, Element},
    prelude::*,
    query::Query,
    request::{Body, Head, Headers, Method},
    server::{
        router::{get, post},
        Router, Server, Socket,
    },
    Request,
};

use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
struct Quote {
    id: Option<String>,
    content: String,
    author: String,
    tags: Vec<String>,
    authorSlug: String,
    length: u16,
    dateAdded: String,
    dateModified: String,
}

/// Credit to: https://github.com/lukePeavey/quotable
/// This is the api used for getting quotes
async fn random_quote(mut cookies: Cookies) -> Element {
    let response = Request::builder()
        .uri("https://api.quotable.io/random")
        .send()
        .await;

    if let Some(_) = cookies.get("One") {
        cookies.delete("One");
        cookies.set("Two", Cookie::new("2"));
    } else {
        cookies.delete("Two");
        cookies.set("One", Cookie::new("1"));
    }

    let quote: Quote = response.json().await.unwrap();
    println!("Author: {}", quote.author);
    html::new! {
        <blockquote>
            <em>{quote.content}</em>
            <br/>
            <strong>"- "{quote.author}</strong>
        </blockquote>
    }
}

async fn home(mut cookies: Cookies) -> Element {
    cookies.set("count", Cookie::new(0));

    html::new! {
        <html>
            <head>
                <title>"Featured Example"</title>
                <style>r#"
                    body {
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;

                        min-height: 100vh;
                        min-height: 100dvh;
                        margin: 0;
                        padding: 0;
                    }
                    #quote {
                        width: fit-content;
                        max-width: 90ch;
                    }
                    blockquote strong {
                        display: block;
                        margin-top: 1rem;
                        font-size: 0.875rem;
                    }
                    blockquote em {
                        font-size: 1.3rem;
                        text-align: center;
                    }
                    blockquote em::before {
                        content: "\""
                    }
                    blockquote em::after {
                        content: "\""
                    }
                "#</style>
                <script>r#"
                    window.onload = async () => {
                        let quoteInput = document.getElementById('quote');

                        const response = await fetch('/', { method: "POST" });

                        const text = await response.text();
                        if (response.status === 200) {
                            if (quoteInput) {
                                quoteInput.innerHTML = text;
                            }
                        }
                    };
                "#</script>
            </head>
            <body>
                <!-- "Credit to: https://github.com/lukePeavey/quotable for the quotes api" -->
                <div id="quote">
                </div>
            </body>
        </html>
    }
}

/// Run this example with the macros and log features
#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving to {}", addr))
        .serve(
            Socket::Local(3000),
            Router::new()
                .route("/", get(home))
                .route("/", post(random_quote)),
        )
        .await;
}
