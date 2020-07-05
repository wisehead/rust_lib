use std::io::prelude::*;
use std::fs::OpenOptions;

fn main() -> std::io::Result<()> {
   
    let mut file = OpenOptions::new()
            .append(true).open("./text.txt")?;

    file.write(b" APPEND WORD")?;

    Ok(())
}
