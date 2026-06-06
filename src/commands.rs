pub enum Command {
    Exit,
    Echo(String),
}

#[derive(thiserror::Error, Debug)]
#[error("{command}: command not found")]
pub struct ParseError {
    command: String,
}

impl Command {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim();
        let (command, args) = input.split_once(' ').unwrap_or((input, ""));

        let command = command.to_lowercase();
        let args = args.trim_start();

        match command.as_str() {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo(args.into())),
            _ => Err(ParseError {
                command: command.into(),
            }),
        }
    }
}
