extern crate tela;
use tela::response::html;

fn main() {
    let data = 33;
    println!("{}", html::string!{
        <p {data}>"data"</p>
    })
}
