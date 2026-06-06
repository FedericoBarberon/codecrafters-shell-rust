#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("failed to read command");

    let command = buf.trim();
    println!("{command}: command not found");
}
