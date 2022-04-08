//! Simple Environment variable setter which loads a specified file,
//! reads the key value pairs from each new line, and inserts them into the runtime environment.
//! example:
//!
//! `EnvironmentLoader::new(file_path.to_str().unwrap());`
//!
//! Where `file_path` is a `&str` path to the desired env to load, and its content is as follows:
//!
//! ```
// E1=123
// E2=ABC
//! ```
//! The result would be environment variables (`E1`, `E2`) being set with the specified values after the first `=` sign.
//!

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[cfg(test)]
mod tests {
    extern crate tempdir;
    use tempdir::TempDir;

    use crate::EnvironmentLoader;
    use std::{env, fs::File, io::Write, path::PathBuf};

    struct TempDirContext {
        file_path: PathBuf,
        _dir: TempDir, // just to keep context per test run
    }

    fn tmp_env_file(env_str: &str) -> TempDirContext {
        let _dir = tempdir::TempDir::new("test-env").unwrap();
        let fname = "foo.txt".to_string();
        let file_path = _dir.path().join(&fname);
        let mut f = File::create(&file_path).unwrap();
        f.write_all(env_str.as_bytes()).unwrap();
        TempDirContext { file_path, _dir }
    }

    #[test]
    fn multi_equal_single_line() {
        let valid_env = "ENV1=test=test===test3";
        let context = tmp_env_file(valid_env);
        let file_path = context.file_path;
        EnvironmentLoader::new(file_path.to_str().unwrap());
        let e1 = env::var("ENV1");
        assert_eq!(e1.unwrap(), "test=test===test3");
    }
    #[test]
    fn preserves_single_quotes() {
        let valid_env = "ENV2='as2d'";
        let context = tmp_env_file(valid_env);
        let file_path = context.file_path;
        EnvironmentLoader::new(file_path.to_str().unwrap());
        let e3 = env::var("ENV2");
        assert_eq!(e3.unwrap(), "'as2d'");
    }
    #[test]
    fn loads_valid_env() {
        let valid_env = "ENV3=test\nENV4=1234";
        let context = tmp_env_file(valid_env);
        let file_path = context.file_path;
        EnvironmentLoader::new(file_path.to_str().unwrap());
        let e1 = env::var("ENV3");
        let e2 = env::var("ENV4");
        assert_eq!(e1.unwrap(), "test");
        assert_eq!(e2.unwrap(), "1234");
    }
}

/// Loads key value pairs from a file into the programs environment
pub struct EnvironmentLoader {}
impl EnvironmentLoader {
    pub fn new(file_path: &str) {
        let file = File::open(file_path).expect("file not found!");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line_str = &line.unwrap();
            EnvironmentLoader::to_env(line_str);
        }
    }
    fn to_env(env_line: &String) {
        let mut split = env_line.splitn(2, "=");
        let key = split.next().unwrap();
        let val = split.next().unwrap();
        if env::var_os(key) == None {
            env::set_var(key, val);
        }
    }
}
