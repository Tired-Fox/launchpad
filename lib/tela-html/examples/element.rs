use tela_html::element::Element;

fn main() {
    let element = Element::tag(
        "div",
        [("hidden", "yes"), ("title", "Hello, world!")],
        vec![
            Element::text("Some text here"),
            Element::comment("Random comment"),
            Element::tag("p", None, vec![Element::text("Nested text")]),
        ],
    );
    println!("{:?}\n", element);
    println!("{}", element);
}
