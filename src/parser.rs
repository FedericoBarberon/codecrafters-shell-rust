use crate::command_node::CommandNode;

#[derive(Debug, PartialEq)]
pub struct RawCommand {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("Failed to parse, input cannot be empty")]
    EmptyInput,
}

pub fn parse(input: &str) -> Result<CommandNode<RawCommand>, ParseError> {
    let tokens = input.split_whitespace().collect::<Vec<&str>>();

    if tokens.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let command = tokens[0].to_lowercase();
    let args = if tokens.len() > 1 {
        tokens[1..].into_iter().map(|&s| String::from(s)).collect()
    } else {
        Vec::new()
    };

    Ok(CommandNode::Single(RawCommand { command, args }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_with_args() {
        let input = "echo hello world!";
        let out = parse(input).unwrap();
        let expected = CommandNode::Single(RawCommand {
            command: "echo".into(),
            args: build_args(&["hello", "world!"]),
        });

        assert_eq!(out, expected);
    }

    #[test]
    fn command_without_args() {
        let input = "ls";
        let out = parse(input).unwrap();
        let expected = CommandNode::Single(RawCommand {
            command: "ls".into(),
            args: Vec::new(),
        });

        assert_eq!(out, expected);
    }

    #[test]
    fn normalize_command_name() {
        let input = "CommAnD with Args NotNormalized";
        let out = parse(input).unwrap();
        let expected = CommandNode::Single(RawCommand {
            command: "command".into(),
            args: build_args(&["with", "Args", "NotNormalized"]),
        });

        assert_eq!(out, expected);
    }

    #[test]
    fn empty_command() {
        let input = "";
        let out = parse(input);

        std::assert_eq!(out, Err(ParseError::EmptyInput));
    }

    fn build_args(args: &[&str]) -> Vec<String> {
        args.into_iter().map(|&arg| String::from(arg)).collect()
    }
}
