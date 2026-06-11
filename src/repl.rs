use std::io::{self, Write};

use crate::{
    executor::{ExecutionResult, execute},
    parser::parse,
    resolver::resolve_command,
};

pub fn start() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .expect("failed to read command");

        let input = buf.trim();

        let raw_command_node = match parse(input) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let command_node = match resolve_command(raw_command_node) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        match execute(command_node, &mut io::stdout(), &mut io::stderr()) {
            Ok(ExecutionResult::Continue) => continue,
            Ok(ExecutionResult::Exit) => break,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        }
    }
}
