extern crate tela;
use tela::html::{self, props, Element, Props};

/// Async component
///
/// The macro will call await on the macro if the `await` attribute is provided in the markup.
/// However, you will still need a async runtime like `tokio` to execute the await.
async fn component(props: Props) -> Element {
    println!("{:?}", props);

    html::new! {
        <div>"From async"</div>
    }
}

#[tokio::main]
async fn main() {
    let data = 33;
    let d = [1, 2, 3, 4, 5];

    let attrs = props! {
        data: data,
        name: "tela"
    };

    println!(
        "{}",
        html::new! {
            <p {data} {..attrs}>
            {"<script>const _ = 'auto escaped'</script>"}
            </p>
            <!-- "The await in the `for` element will await the async handler that is it's child" -->
            <for let:d await>
                {|text: u8| async move {
                    // The element here has the await attribute. This will call await on the result
                    // of executing the component.
                    html::new! {
                        <component text={text} await/>
                    }
                }}
            </for>
        }
        // Need to explicitly convert to a String as `Display` is not implemented for an html
        // element
        .to_string()
    )
}
