extern crate wayfinder;

use wayfinder::{
    prelude::*,
    response::{
        template::{Handlebars, Tera},
        Template,
    },
    Server,
};

#[get("/")]
fn home() -> Template<Tera> {
    template!("index.html", { title: "Tera" })
    // Equal to:
    // Template::<Tera>::new(
    //      "index.html".to_string(),
    //      BTreeMap<String, serde_json::Value>::from([("title", "Tera")]
    // )
}

#[get("/blog")]
fn blog() -> Template<Handlebars> {
    template!("blog.html", { ...Handlebars::globals(), title: "Handlebars" })
    // Equal to:
    // Template::<Handlebars>::new(
    //      "blog.html".to_string(),
    //      {
    //          let mut __temp = Handlebars::globals();
    //          __temp.append(BTreeMap<String, serde_json::Value>::from([("title",
    //          "Handlebars")]));
    //          __temp
    //      }
    // )
}

/// Run `cargo run --example templates --features=tera,handlebars`
/// The templating engines are initialized with the server and create a slightly longer
/// startup and initial build time.
///
/// The `context!` macro is a way of constructing BTreeMaps. These are used for the variables being
/// exposed to the templating engines. The macro allows for "spreading" a BTreeMap, which in
/// reality just appends all the other values to the spread BTreeMap.
///
/// The `template!` macro allows for easy template construction. The macro expands to use the
/// context macro around whatever is inside of the curly brackets. It will also take the path to
/// the template relative to the root template path provided in the server builder method.
///
/// The server builder methods `tera` and `handlebars` serve to initialize the templating engines
/// along with providing the root template path, along with global variables. The first argument is
/// the path to the templates while the second is any BTreeMap<String, serde_json::Value> of global
/// values. The `context!` macro works great for creating this map.
#[wayfinder::main]
async fn main() {
    Server::new()
        .tera("examples/assets/templates/", context! {})
        .handlebars(
            "examples/assets/templates/",
            context! { message: "Hello world!" },
        )
        .route(home)
        .route(blog)
        .serve(3000)
        .await
}
