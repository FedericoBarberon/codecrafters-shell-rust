use std::io::{self, Write};

use crate::{
    executor::{ExecutionResult, execute},
    parser::ParsedCommand,
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

        let parsed_command = ParsedCommand::parse(input);

        if let Err(_) = parsed_command {
            continue;
        }

        let parsed_command = parsed_command.unwrap();
        let command = resolve_command(parsed_command);

        if let Err(e) = command {
            eprintln!("{e}");
            continue;
        }

        let command = command.unwrap();

        match execute(command, &mut io::stdout(), &mut io::stderr()) {
            Ok(ExecutionResult::Continue) => continue,
            Ok(ExecutionResult::Exit) => break,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        }
    }
}
