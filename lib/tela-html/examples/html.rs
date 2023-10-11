extern crate tela_html;
use tela_html::{element::Element, html, props, Props};

fn component(props: Props) -> Element {
    println!("{:?}", props);
    let data = [("today", "Wednesday"), ("tomorrow", "Thursday")];
    html! {
        <div>{props.children()}</div>
        <ul>
        <for let:data>
            <!-- "data.iter().map({closure}).collect::<Vec<Element>>()" -->
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
    // [("hidden".to_string(), "true".to_string()), ("alt".to_string(), "Wrapper".to_string())];

    let result = html! {
        <!-- "Some Comment" -->
        <div title="something" {..props! {
            hidden: true,
            alt: "Wrapper"
        }}>
            <p>
                "Some text here"
            </p>
            <component {..attrs}>
                { error_code }
            </component>
            { || html! {
                <div>"Hello, world!"</div>
            }}
        </div>
    };

    println!("{:?}", result);
    println!("{}", result);
}
