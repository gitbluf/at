use std::{
    fmt,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

#[derive(Debug)]
pub enum TerraError {
    InvalidPath(PathBuf),
    CommandFailed(String),
    InvalidArgs,
}

impl fmt::Display for TerraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerraError::InvalidPath(path) => {
                write!(f, "Invalid Terraform path: {}", path.display())
            }
            TerraError::CommandFailed(output) => {
                write!(f, "{}", output)
            }
            TerraError::InvalidArgs => write!(f, "No arguments provided"),
        }
    }
}

impl std::error::Error for TerraError {}

#[derive(Debug)]
pub struct TerraExecutor {
    terra_path: PathBuf,
}

impl TerraExecutor {
    pub fn new(terra_path: impl AsRef<Path>) -> Result<Self, TerraError> {
        let path = PathBuf::from(terra_path.as_ref());

        if !path.exists() {
            return Err(TerraError::InvalidPath(path.clone()));
        }

        Ok(Self { terra_path: path })
    }

    pub fn execute_command(&self, args: &[String]) -> Result<String, TerraError> {
        if args.is_empty() {
            return Err(TerraError::InvalidArgs);
        }

        let mut cmd = Command::new(&self.terra_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = cmd.stdout.take().expect("failed to open stdout");
        let stderr = cmd.stderr.take().expect("failed to open stderr");
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.unwrap_or_default();
                tx_clone.send(format!("{}", line)).unwrap();
            }
        });

        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = line.unwrap_or_default();
                tx.send(format!("{}", line)).unwrap();
            }
        });

        let mut output = String::new();
        for line in rx {
            println!("{}", line);
            output.push_str(&line);
            output.push('\n');
        }

        let status = cmd
            .wait()
            .map_err(|e| TerraError::CommandFailed(e.to_string()))?;

        if !status.success() {
            return Err(TerraError::CommandFailed(output));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_path() {
        let path = "/invalid/path/to/terraform";
        let result = TerraExecutor::new(path);
        assert!(result.is_err());
        match result {
            Err(TerraError::InvalidPath(pb)) => {
                assert_eq!(pb.to_str().unwrap(), path);
            }
            _ => panic!("Expected InvalidPath error, got {:?}", result),
        }
    }

    #[test]
    fn test_execute_command_empty_args() {
        let executor = TerraExecutor::new(".").unwrap();
        let args: Vec<String> = vec![];
        let result = executor.execute_command(&args);
        assert!(result.is_err());
        match result {
            Err(TerraError::InvalidArgs) => {}
            _ => panic!("Expected InvalidArgs error, got {:?}", result),
        }
    }

    #[test]
    fn test_execute_command_success() {
        let executor = TerraExecutor::new("/bin/echo").unwrap();
        let args = vec!["Hello, world!".to_string()];
        let result = executor.execute_command(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello, world!");
    }

    #[test]
    fn test_execute_command_failure() {
        let executor = TerraExecutor::new("/bin/ls").expect("Failed to create executor");
        let result = executor.execute_command(&["--wfafa".to_string()]);
        assert!(result.is_err());
        match result {
            Err(TerraError::CommandFailed(err_msg)) => {
                assert!(!err_msg.is_empty());
            }
            _ => panic!("Expected CommandFailed error, got {:?}", result),
        }
    }
}
