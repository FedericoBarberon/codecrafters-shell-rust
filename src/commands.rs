use std::{
    env,
    io::{Read, Write},
    path::PathBuf,
    process,
};

use strum_macros::{AsRefStr, EnumIter, EnumString};

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
    BuiltIn(BuiltInKind),
    External { path: PathBuf },
}

#[derive(Debug, PartialEq)]
pub enum BuiltIn {
    Exit,
    Echo { args: Vec<String> },
    Type { args: Vec<String> },
    Pwd,
    Cd { args: Vec<String> },
}

#[derive(Debug, EnumString, PartialEq, EnumIter, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum BuiltInKind {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

impl BuiltInKind {
    pub fn build(self, args: Vec<String>) -> BuiltIn {
        match self {
            Self::Exit => BuiltIn::Exit,
            Self::Echo => BuiltIn::Echo { args },
            Self::Type => BuiltIn::Type { args },
            Self::Pwd => BuiltIn::Pwd,
            Self::Cd => BuiltIn::Cd { args },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct External {
    command: String,
    path: PathBuf,
    args: Vec<String>,
}

impl External {
    pub fn new(command: String, path: PathBuf, args: Vec<String>) -> Self {
        Self {
            command,
            path,
            args,
        }
    }
}

impl Executable for External {
    fn execute(
        &self,
        _in_buf: &mut impl Read,
        _out_buf: &mut impl Write,
        _err_buf: &mut impl Write,
    ) -> Result<ExecutionResult, ExecutionError> {
        let _status = process::Command::new(&self.command)
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
                        Some(CommandType::BuiltIn(_)) => {
                            writeln!(out_buf, "{arg} is a shell builtin")
                        }
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
            BuiltIn::Cd { args } => {
                if args.is_empty() {
                    let _ = writeln!(err_buf, "cd: too few arguments");
                } else if args.len() > 1 {
                    let _ = writeln!(err_buf, "cd: too many arguments");
                } else {
                    let path = PathBuf::from(&args[0]);

                    if path.is_dir() {
                        if let Err(e) = env::set_current_dir(path) {
                            let _ = writeln!(err_buf, "cd: failed to change directory: {e}");
                        }
                    } else {
                        let _ = writeln!(
                            err_buf,
                            "cd: {}: No such file or directory",
                            path.to_string_lossy()
                        );
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
