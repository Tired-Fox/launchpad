use std::fs;

fn main() {
    let data = fs::read_to_string("web/getData.js").expect("Could not read from file");
    println!("{}", data)
}
