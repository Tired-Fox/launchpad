use tela::html::{Element, Props, html};

mod test {
    use tela::html::{Element, Props, html};
    pub fn data(props: Props) -> Element {
        html !{
            <div>"Data"</div>
        }
    }
}

fn main() {
    println!("{}",
        html!{
            <test::data />
        }.to_string()
    );
}
