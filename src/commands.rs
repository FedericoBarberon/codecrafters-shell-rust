use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
    process,
};

use crate::{
    executor::{Executable, ExecutionError, ExecutionResult},
    resolver::lookup,
};

#[derive(Debug, PartialEq)]
pub enum Command {
    BuiltIn(BuiltIn),
    External(External),
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    BuiltIn,
    External { path: PathBuf },
}

#[derive(Debug, PartialEq)]
pub enum BuiltIn {
    Exit,
    Echo { args: Vec<String> },
    Type { args: Vec<String> },
    Pwd,
}

#[derive(Debug, PartialEq)]
pub struct External {
    path: PathBuf,
    args: Vec<String>,
}

impl External {
    pub fn new(path: PathBuf, args: Vec<String>) -> Self {
        Self { path, args }
    }
}

impl Executable for External {
    fn execute(
        &self,
        _in_buf: &mut impl Read,
        _out_buf: &mut impl Write,
        _err_buf: &mut impl Write,
    ) -> Result<ExecutionResult, ExecutionError> {
        let _status = process::Command::new(&self.path)
            .args(&self.args)
            .status()
            .map_err(|error| ExecutionError {
                error: error.to_string(),
            })?;

        Ok(ExecutionResult::Continue)
    }
}

impl Executable for BuiltIn {
    fn execute(
        &self,
        _in_buf: &mut impl Read,
        out_buf: &mut impl Write,
        err_buf: &mut impl Write,
    ) -> Result<ExecutionResult, ExecutionError> {
        match self {
            BuiltIn::Exit => Ok(ExecutionResult::Exit),
            BuiltIn::Echo { args } => {
                let _ = writeln!(out_buf, "{}", args.join(" "));
                Ok(ExecutionResult::Continue)
            }
            BuiltIn::Type { args } => {
                if args.is_empty() {
                    let _ = writeln!(err_buf, "type: too few arguments");
                    return Ok(ExecutionResult::Continue);
                }

                for arg in args {
                    let _ = match lookup(&arg) {
                        Some(CommandType::BuiltIn) => writeln!(out_buf, "{arg} is a shell builtin"),
                        Some(CommandType::External { path }) => {
                            writeln!(out_buf, "{arg} is {}", path.to_string_lossy())
                        }
                        None => writeln!(out_buf, "{arg} not found"),
                    };
                }
                Ok(ExecutionResult::Continue)
            }
            BuiltIn::Pwd => {
                match env::current_dir() {
                    Ok(cd) => {
                        let _ = writeln!(out_buf, "{}", cd.to_string_lossy());
                    }
                    Err(e) => {
                        let _ = writeln!(err_buf, "{e}");
                    }
                }
                Ok(ExecutionResult::Continue)
            }
        }
    }
}

impl Executable for Command {
    fn execute(
        &self,
        in_buf: &mut impl Read,
        out_buf: &mut impl Write,
        err_buf: &mut impl Write,
    ) -> Result<ExecutionResult, ExecutionError> {
        match self {
            Command::BuiltIn(b) => b.execute(in_buf, out_buf, err_buf),
            Command::External(e) => e.execute(in_buf, out_buf, err_buf),
        }
    }
}
