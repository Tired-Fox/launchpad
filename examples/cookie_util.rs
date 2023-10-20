use tela::{
    cookie::{Cookie, CookieJar, Duration, Local},
    html,
    server::{router::get, Router, Server, Socket},
};

const TELA_COOKIE: &'static str = "TelaCookie";

async fn handler(mut cookies: CookieJar) -> html::Element {
    match cookies.get(TELA_COOKIE) {
        Some(_) => cookies.delete(TELA_COOKIE),
        None => cookies.set(
            TELA_COOKIE,
            Cookie::builder(1)
                .expires(Local::now() + Duration::minutes(1))
                .max_age(60)
                .finish(),
        ),
    };

    html::new! {
        <style>r#"
            code {
                padding-block: .12rem;
                padding-inline: .24rem;
                background-color: #494c52;
                color: white;
                border-radius: .2rem;
            }
            body {
                padding: 0;
                margin: 0;
                box-sizing: border-box;
                width: 100vw;
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
            }
        "#</style>
        <p>"Check "<code>"ctrl+shift+I"</code>" > "<code>"Application"</code>" > "<code>"Cookies"</code>" for the cookies being update on each page load"</p>
    }
}

#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving to {}", addr))
        .serve(
            Socket::Local(3000),
            Router::builder().route("/", get(handler)),
        )
        .await;
}
