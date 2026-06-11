use crate::command_node::CommandNode;
use std::io;
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
pub enum ExecutionResult {
    Continue,
    Exit,
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("{error}")]
pub struct ExecutionError {
    pub error: String,
}

pub trait Executable {
    fn execute(
        &self,
        in_buf: &mut impl Read,
        out_buf: &mut impl Write,
        err_buf: &mut impl Write,
    ) -> Result<ExecutionResult, ExecutionError>;
}

pub fn execute<C: Executable>(
    command_node: CommandNode<C>,
    out_buf: &mut impl Write,
    err_buf: &mut impl Write,
) -> Result<ExecutionResult, ExecutionError> {
    match command_node {
        CommandNode::Single(command) => command.execute(&mut io::stdin(), out_buf, err_buf),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{commands::Command, parser::parse, resolver::resolve_command};

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

    fn build_command(input: &str) -> CommandNode<Command> {
        resolve_command(parse(input).unwrap()).unwrap()
    }

    fn test_execute(
        input: &str,
        expected_out: Option<&str>,
        expected_err: Option<&str>,
        expected_result: Result<ExecutionResult, ExecutionError>,
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
