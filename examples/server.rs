extern crate tela;

use tela::{
    html::{self, Html},
    prelude::*,
    request::Body,
    server::{router::get, Router, Server, StatusCode},
};

async fn not_found() -> Html<String> {
    // html::from will convert to HTML<String> while
    // html::new! will convert to tela::html::Element.
    // Either may be returned
    html::from! {
        <h1>{StatusCode::NOT_FOUND}</h1>
    }
}

async fn hours() -> Html<String> {
    html::from! {
        <html>
            <head>
                <script>r#"
                    async function getPost() {
                        let tbox = document.getElementById('post-result');

                        if (tbox) {
                            tbox.value = '';
                            tbox.value = await (await fetch('/hours',{method: 'POST', body: JSON.stringify({name: 'Tela'})})).text();
                        } else {
                            console.error('Failed to find result text box');
                        }
                    }
                "#</script>
                <title>"Example"</title>
                <style>r#"
                    button {margin-bottom: 1rem;}
                    label {display: block;}
                "#</style>
            </head>
            <body>
                <h1>"Make A Request"</h1>
                <img src="/images/cat-lounge.jpg" alt="Lounging Cat" />
                <br />
                <button type="button" onclick="getPost()">"POST Request"</button>
                <label for="post-result">"Result"</label>
                <textarea type="text" id="post-result" disabled title="Result"></textarea>
            </body>
        </html>
    }
}

#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving to {}", addr))
        .serve(
            socket!(3000, 4000),
            Router::builder()
                .assets(("/images/", "examples/assets/"))
                .route(
                    "/hours",
                    get(hours)
                        .post(|body: Body| async {
                            match body.text().await {
                                Ok(body) => println!("{}", body),
                                Err(e) => eprintln!("{}", e),
                            }
                            "Post request works!"
                        })
                        .fallback(not_found),
                )
                .any(not_found),
        )
        .await;
}
