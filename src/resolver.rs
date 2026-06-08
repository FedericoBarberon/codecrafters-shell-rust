use std::{env, path::PathBuf};

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
    match lookup(&command) {
        Some(CommandType::BuiltIn) => resolve_builtin(command, args),
        Some(CommandType::External { path: _ }) => Ok(Command::External { command, args }),
        None => Err(ResolveError::UnknownCommand { command }),
    }
}

fn resolve_builtin(command: String, args: Vec<String>) -> Result<Command, ResolveError> {
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
        _ => unreachable!(),
    }
}

pub fn lookup(command: &str) -> Option<CommandType> {
    if is_builtin(command) {
        Some(CommandType::BuiltIn)
    } else if let Some(path) = find_binary(command, &env::var("PATH").ok()?) {
        Some(CommandType::External { path })
    } else {
        None
    }
}

fn is_builtin(command: &str) -> bool {
    ["echo", "exit", "type"].contains(&command)
}

fn find_binary(name: &str, paths: &str) -> Option<PathBuf> {
    for mut path in env::split_paths(paths) {
        path.push(name);

        if path.is_file() {
            #[cfg(unix)]
            {
                use permissions::is_executable;

                if is_executable(&path).unwrap_or(false) {
                    return Some(path);
                }
            }

            #[cfg(windows)]
            {
                return Some(path);
            }
        }
    }

    None
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

    #[test]
    fn lookup_builtin() {
        let commands = ["echo", "exit", "type"];

        for cmd in commands {
            assert_eq!(lookup(cmd), Some(CommandType::BuiltIn));
        }
    }

    #[test]
    fn lookup_unknown() {
        assert_eq!(lookup("unknown_command_foo"), None);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[cfg(unix)]
        mod unix {
            use super::*;
            use std::fs;
            use std::os::unix::fs::PermissionsExt;
            use std::path::{Path, PathBuf};
            use tempfile::tempdir;

            fn create_file(dir: &Path, name: &str, executable: bool) -> PathBuf {
                let path = dir.join(name);

                fs::write(&path, "").unwrap();

                let mut perms = fs::metadata(&path).unwrap().permissions();
                perms.set_mode(if executable { 0o755 } else { 0o644 });
                fs::set_permissions(&path, perms).unwrap();

                path
            }

            fn make_path(paths: &[&Path]) -> String {
                std::env::join_paths(paths)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            }

            #[test]
            fn finds_binary_in_first_directory() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let binary = create_file(dir1.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(binary));
            }

            #[test]
            fn finds_binary_in_last_directory() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let binary = create_file(dir2.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(binary));
            }

            #[test]
            fn returns_none_when_binary_does_not_exist() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), None);
            }

            #[test]
            fn returns_first_match_when_binary_exists_multiple_times() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let first = create_file(dir1.path(), "mycmd", true);
                create_file(dir2.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(first));
            }

            #[test]
            fn ignores_non_executable_files() {
                let dir = tempdir().unwrap();

                create_file(dir.path(), "mycmd", false);

                let path = make_path(&[dir.path()]);

                assert_eq!(find_binary("mycmd", &path), None);
            }

            #[test]
            fn skips_non_executable_match_and_finds_executable_one() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                create_file(dir1.path(), "mycmd", false);
                let executable = create_file(dir2.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(executable));
            }

            #[test]
            fn ignores_directories_with_matching_name() {
                let dir = tempdir().unwrap();

                fs::create_dir(dir.path().join("mycmd")).unwrap();

                let path = make_path(&[dir.path()]);

                assert_eq!(find_binary("mycmd", &path), None);
            }
        }

        #[cfg(windows)]
        mod windows {
            use super::*;
            use std::env;
            use std::ffi::OsString;
            use std::fs;
            use std::path::{Path, PathBuf};
            use tempfile::tempdir;

            fn create_file(dir: &Path, name: &str, executable: bool) -> PathBuf {
                let filename = if executable {
                    format!("{name}.exe")
                } else {
                    name.to_string()
                };

                let path = dir.join(filename);

                fs::write(&path, "").unwrap();

                path
            }

            fn make_path(paths: &[&Path]) -> String {
                std::env::join_paths(paths)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            }

            #[test]
            fn finds_binary_in_first_directory() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let binary = create_file(dir1.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(binary));
            }

            #[test]
            fn finds_binary_in_last_directory() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let binary = create_file(dir2.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(binary));
            }

            #[test]
            fn returns_none_when_binary_does_not_exist() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), None);
            }

            #[test]
            fn returns_first_match_when_binary_exists_multiple_times() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                let first = create_file(dir1.path(), "mycmd", true);
                create_file(dir2.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(first));
            }

            #[test]
            fn ignores_non_executable_files() {
                let dir = tempdir().unwrap();

                create_file(dir.path(), "mycmd", false);

                let path = make_path(&[dir.path()]);

                assert_eq!(find_binary("mycmd", &path), None);
            }

            #[test]
            fn skips_non_executable_match_and_finds_executable_one() {
                let dir1 = tempdir().unwrap();
                let dir2 = tempdir().unwrap();

                create_file(dir1.path(), "mycmd", false);
                let executable = create_file(dir2.path(), "mycmd", true);

                let path = make_path(&[dir1.path(), dir2.path()]);

                assert_eq!(find_binary("mycmd", &path), Some(executable));
            }

            #[test]
            fn ignores_directories_with_matching_name() {
                let dir = tempdir().unwrap();

                fs::create_dir(dir.path().join("mycmd.exe")).unwrap();

                let path = make_path(&[dir.path()]);

                assert_eq!(find_binary("mycmd", &path), None);
            }
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
