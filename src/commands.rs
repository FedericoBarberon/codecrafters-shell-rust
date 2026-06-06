pub enum Command {
    Exit,
}

#[derive(thiserror::Error, Debug)]
#[error("{input}: command not found")]
pub struct ParseError {
    input: String,
}

impl Command {
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "exit" => Ok(Self::Exit),
            _ => Err(ParseError { input }),
        }
    }
}
