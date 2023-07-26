extern crate launchpad;
use launchpad::{prelude::*, router::response::HTML, Server};

mod routes;
use routes::{
    api::{data, plain},
    error_page, index, not_found, unexpected,
};

#[get("/test/<info>")]
fn test(info: &str) -> Result<HTML<String>> {
    HTML::ok(html! {
        <h1>"Test Page"</h1>
        <p>"Info: "{info}</p>
    })
}

#[tokio::main]
async fn main() {
    Server::new()
        .router(rts! {
            [ index, error_page, data, plain, test ],
            catch {
                503 => unexpected,
                404 => not_found
            },
        })
        .serve(3000)
        .await;
}
