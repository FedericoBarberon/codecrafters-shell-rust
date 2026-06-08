#[derive(Debug, PartialEq)]
pub enum Command {
    Exit,
    Echo { args: Vec<String> },
    Type { args: Vec<String> },
}

pub enum CommandType {
    BuiltIn,
    External { path: String },
}
