use std::io::prelude::*;
use std::fs::OpenOptions;

fn main() -> std::io::Result<()> {
   
    let mut file = OpenOptions::new()
            .read(true).write(true).open("./text.txt")?;

    file.write(b"COVER")?;

    Ok(())
}
