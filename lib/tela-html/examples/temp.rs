#[macro_use]
extern crate std;
extern crate tela_html;
use tela_html::{element::Element, html, props, Props};
fn Component(props: Props) -> Element {
    format!("{:?}", props);
    let data = [("today", "Wednesday"), ("tomorrow", "Thursday")];
    {
        use tela_html::prelude::*;
        Element::tag("", None, {
            let mut _t: Vec<Element> = Vec::new();
            _t.push(Element::tag("div", None, {
                let mut _t: Vec<Element> = Vec::new();
                match { props.children() }.into_children() {
                    Some(values) => _t.extend(values),
                    None => {}
                };
                _t
            }));
            _t.push(Element::tag("ul", None, {
                let mut _t: Vec<Element> = Vec::new();
                ()
            }));
            _t
        })
    }
}
fn main() {
    let error_code = 34;
    let attrs = [
        ("hidden".replace("_", "-"), true.to_string()),
        ("alt".replace("_", "-"), "Wrapper".to_string()),
    ];
    let result = {
        use tela_html::prelude::*;
        Element::tag("", None, {
            let mut _t: Vec<Element> = Vec::new();
            _t.push(Element::comment("Some Comment"));
            _t.push(Element::tag(
                "div",
                {
                    let mut attrs = [
                        ("hidden".replace("_", "-"), true.to_string()),
                        ("alt".replace("_", "-"), "Wrapper".to_string()),
                    ]
                    .into_attrs();
                    attrs.extend([("title".to_string(), "\"something\"".to_string())]);
                    attrs
                },
                {
                    let mut _t: Vec<Element> = Vec::new();
                    _t.push(Element::tag("p", None, {
                        let mut _t: Vec<Element> = Vec::new();
                        _t.push(Element::text("Some text here"));
                        _t
                    }));
                    _t.push(Component.create_component(attrs.into_attrs(), {
                        let mut _t: Vec<Element> = Vec::new();
                        match { error_code }.into_children() {
                            Some(values) => _t.extend(values),
                            None => {}
                        };
                        _t
                    }));
                    match {
                        || {
                            use tela_html::prelude::*;
                            Element::tag("div", None, {
                                let mut _t: Vec<Element> = Vec::new();
                                _t.push(Element::text("Hello, world!"));
                                _t
                            })
                        }
                    }
                    .into_children()
                    {
                        Some(values) => _t.extend(values),
                        None => {}
                    };
                    _t
                },
            ));
            _t
        })
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", result));
    };
    {
        ::std::io::_print(format_args!("{0}\n", result));
    };
}
