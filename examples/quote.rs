extern crate tela;

use tela::client::SendRequest;
use tela::error::Error;
use tela::server::router::get;
use tela::{
    prelude::*,
    response::{html, HTML},
    server::{serve, Router, Socket},
    Request,
};

use serde::Deserialize;

#[derive(Deserialize)]
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
async fn random_quote(_: Request) -> Result<HTML<String>, Error> {
    let response = Request::builder()
        .uri("https://api.quotable.io/random")
        .send()
        .await;

    let quote = response.json::<Quote>().await?;
    Ok(html::new! {
        <blockquote>
            <em>{quote.content}</em>
            <br/>
            <strong>"- "{quote.author}</strong>
        </blockquote>
    })
}

async fn home(_: Request) -> HTML<String> {
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
                <div id="quote">
                </div>
            </body>
        </html>
    }
}

/// Run this example with the macros and log features
#[tela::main]
async fn main() {
    serve(
        Socket::Local(3000),
        Router::new().route("/", get(home).post(random_quote)),
    )
    .await;
}
