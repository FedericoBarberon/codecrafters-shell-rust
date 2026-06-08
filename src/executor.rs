use crate::{
    commands::{Command, CommandType},
    resolver::lookup,
};
use std::io::Write;

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("Failed to execute command")]
pub struct ExecuteError {}

#[derive(Debug, PartialEq)]
pub enum ExecutionResult {
    Continue,
    Exit,
}

pub fn execute(
    command: Command,
    out: &mut impl Write,
    _err: &mut impl Write,
) -> Result<ExecutionResult, ExecuteError> {
    match command {
        Command::Echo { args } => {
            let _ = writeln!(out, "{}", args.join(" "));
            Ok(ExecutionResult::Continue)
        }
        Command::Exit => Ok(ExecutionResult::Exit),
        Command::Type { args } => {
            for arg in args {
                let _ = match lookup(&arg) {
                    Some(CommandType::BuiltIn) => writeln!(out, "{arg} is a shell builtin"),
                    Some(CommandType::External { path }) => writeln!(out, "{arg} is {path}"),
                    None => writeln!(out, "{arg} not found"),
                };
            }

            Ok(ExecutionResult::Continue)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::ParsedCommand, resolver::resolve_command};

    mod echo_command {
        use super::*;

        #[test]
        fn exec_echo_with_args() {
            test_execute(
                "echo hello world!",
                Some("hello world!\n"),
                None,
                Ok(ExecutionResult::Continue),
            );
        }

        #[test]
        fn exec_echo_without_args() {
            test_execute("echo", Some("\n"), None, Ok(ExecutionResult::Continue));
        }
    }

    mod exit_command {
        use super::*;

        #[test]
        fn exec_exit() {
            test_execute("exit", None, None, Ok(ExecutionResult::Exit));
        }
    }

    mod type_command {
        use super::*;

        #[test]
        fn exec_type() {
            let expected_out = "echo is a shell builtin\nexit is a shell builtin\ntype is a shell builtin\nfoo not found\n";

            test_execute(
                "type echo exit type foo",
                Some(expected_out),
                None,
                Ok(ExecutionResult::Continue),
            );
        }
    }

    fn build_command(input: &str) -> Command {
        resolve_command(ParsedCommand::parse(input).unwrap()).unwrap()
    }

    fn test_execute(
        input: &str,
        expected_out: Option<&str>,
        expected_err: Option<&str>,
        expected_result: Result<ExecutionResult, ExecuteError>,
    ) {
        let command = build_command(input);
        let mut out_buf = Vec::new();
        let mut err_buf = Vec::new();

        assert_eq!(
            execute(command, &mut out_buf, &mut err_buf),
            expected_result
        );

        if let Some(out) = expected_out {
            assert_eq!(String::from_utf8(out_buf).unwrap(), out);
        }

        if let Some(err) = expected_err {
            assert_eq!(String::from_utf8(err_buf).unwrap(), err);
        }
    }
}
