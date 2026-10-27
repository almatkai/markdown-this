use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let cases: Vec<String> = serde_json::from_str(&input).unwrap();
    let results: Vec<_> = cases
        .iter()
        .map(|s| tezbar_markdown_this::convert(s))
        .collect();
    println!("{}", serde_json::to_string(&results).unwrap());
}
