use tela::{
    html::{self, Element, Props},
    server::{methods::get, Router, Server, Socket},
};

fn component_message(props: Props) -> Element {
    html::new! {
        <p>{props.get("message").unwrap()}</p>
    }
}

fn static_reusable() -> Element {
    html::new!( "Any function that returns something that implements "<em>{"tela::html::IntoChildren"}</em>" can be used." )
}

async fn component_message_async(props: Props) -> Element {
    html::new! {
        <p>{props.get("message").unwrap()}</p>
    }
}

async fn home() -> Element {
    let data = ["one", "two", "three"];
    let into_children = "tela::html::IntoChildren";

    html::new! {
        <!-- "Comments are allowed but are not rendered unless the `comments` feature is active" -->

        <h1>"Built-In HTML templating"</h1>
        <p>
            "Allows for templating similar to the "
            <a
                href="https://crates.io/crates/html-to-string-macro"
                target="_blank"
            >
                "html-to-string-macro"
            </a>
            " crate"
        </p>
        <br />

        <component-message message="Additionally you can use components like this one and pass it props." />
        <!-- "Async components require that `await` is called for it's result to be found" -->
        <component-message-async message="Components can also be async like this one." await />

        <br />
        <!-- "The `for` element can be used to iterate over iterable objects using the `let:` binding" -->
        <p>"For loop elements can be used like this."</p>
        <ul>
            <for let:data>
                {|item: &str| {
                    html::new!(<li>{item}</li>)
                }}
            </for>
        </ul>

        <p>"Or asynchronously like this."</p>
        <!-- "Note: the `async` attribute can be provided for the inner handler to become an async closure" -->
        <ul>
            <for let:data async>
                {|item: &'static str| async move {
                    html::new!(<li>{item}</li>)
                }}
            </for>
        </ul>

        <br />
        <p>"Any item that implements "<em>{"tela::html::IntoChildren"}</em>" can be used inside the braced blocks."</p>
        <p>{|| html::new!(
            "This is automatically implemented for most primitive types along with closures as long as the closure's return type implements "
            <em>{into_children}</em>
        )}</p>
        <p>{static_reusable}</p>
        <p>{html::escape(r#"<script src="attack">escape and unescape helper methods are provided but not automatically enforced</script>"#)}". With that in mind, any blocks that are rendered from variables are considered unsafe and should be used with caution."</p>
    }
}

#[tela::main]
async fn main() {
    Server::builder()
        .on_bind(|addr| println!("Serving at {}", addr))
        .serve(Socket::Local(3000), Router::builder().route("/", get(home)))
        .await;
}
