extern crate tela;
use tela::{
    html::{props, Element, Props},
    response::html,
};

async fn component(props: Props) -> Element {
    html::new! {
        <div>"From async"</div>
    }
}

fn main() {
    let data = 33;
    let attrs = props! {
        data: data,
        name: "tela"
    };
    let d = [1, 2, 3, 4, 5];

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
        .to_string()
    )
}
