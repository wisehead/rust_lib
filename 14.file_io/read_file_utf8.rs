use std::fs;

fn main() {
    let content = fs::read("./text.txt").unwrap();
    println!("{:?}", content);
}
