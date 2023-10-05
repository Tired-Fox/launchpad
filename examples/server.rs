extern crate tela;

use tela::client::SendRequest;
use tela::server::router::get;
use tela::{
    prelude::*,
    server::{serve, Router},
    Request,
};

async fn hours(_: Request) -> String {
    html! {
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
    let response = Request::builder().uri("/hours").method("POST").send().await;
    match response.text().await {
        Ok(text) => println!("{}", text),
        Err(e) => eprintln!("{}", e),
    }

    serve(
        socket!(3000, 4000),
        Router::new().route(
            "/hours",
            get(hours).post(|req: Request| async {
                match req.text().await {
                    Ok(body) => println!("{}", body),
                    Err(e) => eprintln!("{}", e),
                }
                "Hours post request!!!"
            }),
        ),
    )
    .await;
}
