extern crate tela;
use tela::{
    html::{props, Element, Props},
    response::html,
};

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
            <for let:d await>
                {|text: u8| async move {
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
