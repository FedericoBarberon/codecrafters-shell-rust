use std::{env, path::PathBuf, str::FromStr};

use permissions::is_executable;

use crate::{
    command_node::CommandNode,
    commands::{BuiltInKind, Command, CommandType, External},
    parser::RawCommand,
};

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ResolveError {
    #[error("{command}: command not found")]
    UnknownCommand { command: String },
}

pub fn resolve_command(
    raw_command_node: CommandNode<RawCommand>,
) -> Result<CommandNode<Command>, ResolveError> {
    match raw_command_node {
        CommandNode::Single(raw_command) => {
            resolve_raw_command(raw_command).map(|cmd| CommandNode::Single(cmd))
        }
    }
}

fn resolve_raw_command(RawCommand { command, args }: RawCommand) -> Result<Command, ResolveError> {
    match lookup(&command) {
        Some(CommandType::BuiltIn(built_in)) => Ok(Command::BuiltIn(built_in.build(args))),
        Some(CommandType::External { path }) => Ok(Command::External(External::new(path, args))),
        None => Err(ResolveError::UnknownCommand { command }),
    }
}

pub fn lookup(command: &str) -> Option<CommandType> {
    if let Ok(built_in) = BuiltInKind::from_str(command) {
        Some(CommandType::BuiltIn(built_in))
    } else if let Some(path) = find_binary(command, &env::var("PATH").ok()?) {
        Some(CommandType::External { path })
    } else {
        None
    }
}

fn find_binary(name: &str, paths: &str) -> Option<PathBuf> {
    for mut path in env::split_paths(paths) {
        path.push(name);

        if path.is_file() && is_executable(&path).unwrap_or(false) {
            return Some(path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::BuiltIn;
    use crate::parser::parse;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

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
            let expected = Ok(CommandNode::Single(Command::BuiltIn(BuiltIn::Exit)));
            test_resolve(input, expected);
        }
    }

    mod echo_command {
        use super::*;

        #[test]
        fn resolve_echo_with_args() {
            let input = "echo hello world!";
            let expected = Ok(CommandNode::Single(Command::BuiltIn(BuiltIn::Echo {
                args: build_args(&["hello", "world!"]),
            })));

            test_resolve(input, expected);
        }
    }

    mod type_command {
        use super::*;

        #[test]
        fn resolve_type_with_args() {
            let input = "type foo bar";
            let expected = Ok(CommandNode::Single(Command::BuiltIn(BuiltIn::Type {
                args: build_args(&["foo", "bar"]),
            })));

            test_resolve(input, expected);
        }
    }

    #[test]
    fn lookup_builtin() {
        use strum::IntoEnumIterator;

        for built_in in BuiltInKind::iter() {
            assert_eq!(
                lookup(built_in.as_ref()),
                Some(CommandType::BuiltIn(built_in))
            );
        }
    }

    #[test]
    fn lookup_unknown() {
        assert_eq!(lookup("unknown_command_foo"), None);
    }

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

    fn test_resolve(input: &str, expected: Result<CommandNode<Command>, ResolveError>) {
        let parsed_command = parse(input).unwrap();
        let out = resolve_command(parsed_command);

        assert_eq!(out, expected);
    }

    fn build_args(args: &[&str]) -> Vec<String> {
        args.into_iter().map(|&arg| String::from(arg)).collect()
    }
}
