use crate::{
    commands::{Command, CommandType},
    parser::ParsedCommand,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ResolveError {
    #[error("{command}: command not found")]
    UnknownCommand { command: String },

    #[error("{command}: too many arguments")]
    TooManyArgs { command: String },

    #[error("{command}: missing arguments")]
    MissingArgs { command: String },
}

pub fn resolve_command(
    ParsedCommand { command, args }: ParsedCommand,
) -> Result<Command, ResolveError> {
    match command.as_str() {
        "exit" => {
            if args.is_empty() {
                Ok(Command::Exit)
            } else {
                Err(ResolveError::TooManyArgs { command })
            }
        }
        "echo" => Ok(Command::Echo { args }),
        "type" => {
            if args.is_empty() {
                Err(ResolveError::MissingArgs { command })
            } else {
                Ok(Command::Type { args })
            }
        }
        _ => Err(ResolveError::UnknownCommand { command }),
    }
}

pub fn lookup(input: &str) -> Option<CommandType> {
    match input {
        "echo" | "exit" | "type" => Some(CommandType::BuiltIn),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_command() {
        let input = "unknown_command foo bar";
        let expected = Err(ResolveError::UnknownCommand {
            command: "unknown_command".into(),
        });
        test_resolve(input, expected);
    }

    mod exit_command {
        use super::*;

        #[test]
        fn resolve_exit() {
            let input = "exit";
            let expected = Ok(Command::Exit);
            test_resolve(input, expected);
        }

        #[test]
        fn exit_invalid_args() {
            let input = "exit foo";
            let expected = Err(ResolveError::TooManyArgs {
                command: "exit".into(),
            });
            test_resolve(input, expected);

            let input = "exit foo bar";
            let expected = Err(ResolveError::TooManyArgs {
                command: "exit".into(),
            });
            test_resolve(input, expected);
        }
    }

    mod echo_command {
        use super::*;

        #[test]
        fn resolve_echo_with_args() {
            let input = "echo hello world!";
            let expected = Ok(Command::Echo {
                args: build_args(&["hello", "world!"]),
            });

            test_resolve(input, expected);
        }

        #[test]
        fn resolve_echo_without_args() {
            let input = "echo";
            let expected = Ok(Command::Echo { args: vec![] });

            test_resolve(input, expected);
        }
    }

    mod type_command {
        use super::*;

        #[test]
        fn resolve_type_with_args() {
            let input = "type foo bar";
            let expected = Ok(Command::Type {
                args: build_args(&["foo", "bar"]),
            });

            test_resolve(input, expected);
        }

        #[test]
        fn resolve_type_without_args() {
            let input = "type";
            let expected = Err(ResolveError::MissingArgs {
                command: "type".into(),
            });

            test_resolve(input, expected);
        }
    }

    fn test_resolve(input: &str, expected: Result<Command, ResolveError>) {
        let parsed_command = ParsedCommand::parse(input).unwrap();
        let out = resolve_command(parsed_command);

        assert_eq!(out, expected);
    }

    fn build_args(args: &[&str]) -> Vec<String> {
        args.into_iter().map(|&arg| String::from(arg)).collect()
    }
}
