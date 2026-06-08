#[derive(Debug, PartialEq)]
pub struct ParsedCommand {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("Failed to parse, input cannot be empty")]
pub struct EmptyInputError {}

impl ParsedCommand {
    pub fn parse(input: &str) -> Result<Self, EmptyInputError> {
        let tokens = input.split_whitespace().collect::<Vec<&str>>();

        if tokens.len() == 0 {
            return Err(EmptyInputError {});
        }

        let command = tokens[0].to_lowercase();
        let args = if tokens.len() > 1 {
            tokens[1..].into_iter().map(|&s| String::from(s)).collect()
        } else {
            Vec::new()
        };

        Ok(Self { command, args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_with_args() {
        let input = "echo hello world!";
        let out = ParsedCommand::parse(input).unwrap();
        let expected = ParsedCommand {
            command: "echo".into(),
            args: build_args(&["hello", "world!"]),
        };

        assert_eq!(out, expected);
    }

    #[test]
    fn command_without_args() {
        let input = "ls";
        let out = ParsedCommand::parse(input).unwrap();
        let expected = ParsedCommand {
            command: "ls".into(),
            args: Vec::new(),
        };

        assert_eq!(out, expected);
    }

    #[test]
    fn normalize_command_name() {
        let input = "CommAnD with Args NotNormalized";
        let out = ParsedCommand::parse(input).unwrap();
        let expected = ParsedCommand {
            command: "command".into(),
            args: build_args(&["with", "Args", "NotNormalized"]),
        };

        assert_eq!(out, expected);
    }

    #[test]
    fn empty_command() {
        let input = "";
        let out = ParsedCommand::parse(input);

        std::assert_eq!(out, Err(EmptyInputError {}));
    }

    fn build_args(args: &[&str]) -> Vec<String> {
        args.into_iter().map(|&arg| String::from(arg)).collect()
    }
}
