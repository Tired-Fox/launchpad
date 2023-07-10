macro_rules! router {
    [$($endpoint: ident),*] => {
        println!("[{}]", vec![$($endpoint.to_string(),)*].join(", "))
    };
    {$($path: literal => $endpoint: ident),*} => {
        println!(
            "[{}]",
            vec![$(($path, $endpoint.to_string()),)*]
                .iter()
                .map(|t| format!("{} => {}", t.0, t.1))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

fn main() {
    let here = "today";
    let something = "something";

    router![here, something];
    router!{
        "Something" => here
    };
}
