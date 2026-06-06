#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .expect("failed to read command");

        let command = buf.trim();

        if command == "exit" {
            break;
        }

        println!("{command}: command not found");
    }
}
