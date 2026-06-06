use codecrafters_shell::commands::Command;
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

        match Command::parse(&buf) {
            Ok(Command::Exit) => break,
            Ok(Command::Echo(msg)) => println!("{msg}"),
            Err(e) => println!("{e}"),
        }
    }
}
