extern crate tela_html;
use std::collections::HashMap;

use tela_html::{element::Element, html, props, Props};

fn component(props: Props) -> Element {
    println!("{:?}", props);
    // Props::fetch will attempt to convert the props to a specified type that implements
    // Deserilize from serde.
    let hidden: bool = props.fetch("hidden").unwrap_or(false);
    let test: HashMap<&str, &str> = props.fetch("data-test").unwrap();

    // The safe varient is Props::get where the raw prop is retrieved as a String. It returns an
    // Option<String> as the key may be missing
    let data_title = props.get("data-title").unwrap_or("".to_string());

    println!(
        "hidden={}\ndata-title={:?}\ntest={:?}",
        hidden, data_title, test
    );

    let data = [("today", "Wednesday"), ("tomorrow", "Thursday")];
    html! {
        <ul data-title={data_title}>
        <for let:data>
            <!-- "data.iter().map({closure}).collect::<Vec<Element>>() ..." -->
            {|(key, value)| html! {
               <li>{key}": "{value}</li>
            }}
        </for>
        </ul>
    }
}

fn main() {
    let error_code = 34;
    let attrs = props! {
        hidden: true,
        alt: "Wrapper"
    };

    let test = HashMap::from([("hidden", "true"), ("alt", "wrapper")]);
    // [("hidden".to_string(), "true".to_string()), ("alt".to_string(), "Wrapper".to_string())];

    let result = html! {
        <!-- "Some Comment" -->
        <div title="something" data-test={test}>
            <p>
                "Some text here"
            </p>
            <component {..attrs} data-test={test} data-title={if error_code == 34 { "ERROR" } else {"SUCCESS"}}>
                { error_code }
            </component>
            { || html! {
                <div>"Hello, world!"</div>
            }}
        </div>
    };

    // println!("{:?}", result);
    println!("{}", result);
}
