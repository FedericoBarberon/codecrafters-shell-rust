use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Command {
    Exit,
    Echo { args: Vec<String> },
    Type { args: Vec<String> },
    External { command: String, args: Vec<String> },
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    BuiltIn,
    External { path: PathBuf },
}
