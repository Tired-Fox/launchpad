extern crate tela;
use tela::{html::props, response::html};

fn main() {
    let data = 33;
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
        }
        .to_string()
    )
}
