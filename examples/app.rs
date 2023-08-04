extern crate wayfinder;
mod routes;

// Routes are defined elsewhere
use routes::{error, files, home, json, redirect, uri_capture};

// Everything needed to create and start the server
use wayfinder::{
    prelude::*,
    response::{
        template::{Handlebars, Tera},
        Template,
    },
    Server, StripPath,
};

#[get("/tera")]
fn template_tera() -> Template<Tera> {
    template!(
        "home.html",
        {...Tera::globals(), "name": "Zachary"}
    )
}

#[get("/handlebars")]
fn template_handlebars() -> Template<Handlebars> {
    template!(
        "home.html",
        {...Handlebars::globals(), "name": "Zachary"}
    )
}

// Allows for tokio::main and result response
#[wayfinder::main]
async fn main() {
    // Tera::init("examples/assets/templates/", Tera::context());
    // Handlebars::init("examples/assets/templates/", Handlebars::context());

    // let tera: Template<Tera> = template!("home.html", {"name": "Zachary"});
    // let hbs: Template<Handlebars> = template!("home.hbs", {"name": "Zachary"});
    //
    // println!("{:?}", tera.render());
    // println!("{:?}", hbs.render());

    Server::new()
        .assets("examples/assets/")
        .tera("examples/assets/templates/", context! { "age": 23 })
        .handlebars("examples/assets/templates", context! { "age": 23 })
        .routes(group![home, uri_capture])
        .routes(group![template_tera, template_handlebars])
        .routes(group![redirect, error::server])
        .routes(group![json::json_string, json::hello_world])
        .routes(group![
            files::html_file,
            files::text_file,
            files::text_to_html_file,
            files::json_file,
            files::text_to_json_file
        ])
        .catch(error::not_found)
        .serve(3000)
        .await
}
