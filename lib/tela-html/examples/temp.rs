use std::collections::HashMap;

use tela_html::ToProp;

fn main() {
    let hash = HashMap::from([("hidden", "true"), ("alt", "wrapper")]);
    println!("{}", hash.to_prop())
}

