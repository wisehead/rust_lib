use std::fs;

fn main() {
    let text = fs::read_to_string("./text.txt").unwrap();
    println!("{}", text);
}
