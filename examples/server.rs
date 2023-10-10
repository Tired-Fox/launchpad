extern crate tela;

use hyper::StatusCode;
use tela::client::SendRequest;
use tela::response::HTML;
use tela::server::router::{fallback, post};
use tela::{
    prelude::*,
    response::html,
    server::{serve, Router},
    Request,
};

async fn not_found(_: Request) -> HTML<String> {
    html::new! {
        <h1>{StatusCode::NOT_FOUND}</h1>
    }
}

async fn hours(_: Request) -> HTML<String> {
    html::new! {
        <html>
            <head>
                <script>r#"
                    async function getPost() {
                        console.log(await (await fetch('/hours',{method: 'POST', body: JSON.stringify({name: 'Tela'})})).text());
                    }
                "#</script>
                <title>"Example"</title>
            </head>
            <body>
                <h1>"GET"</h1>
                <h2>"Hours Page"</h2>
                <button onclick="getPost()">"POST"</button>
            </body>
        </html>
    }
}

#[tela::main]
async fn main() {
    serve(
        socket!(3000, 4000),
        Router::new()
            .route(
                "/hours",
                // get(hours)
                post(|req: Request| async {
                    match req.text().await {
                        Ok(body) => println!("{}", body),
                        Err(e) => eprintln!("{}", e),
                    }
                    "Hours post request!!!"
                })
                .fallback(not_found),
            )
            .fallback(not_found),
    )
    .await;
}
