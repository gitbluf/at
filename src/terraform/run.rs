use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

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
                write!(f, "Command failed with output:\n{}", output)
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

        let mut cmd = Command::new(&self.terra_path);

        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd
            .output()
            .map_err(|e| TerraError::CommandFailed(format!("Failed to execute command: {}", e)))?;

        let stdout = match String::from_utf8(output.stdout) {
            Ok(text) => text,
            Err(e) => {
                return Err(TerraError::CommandFailed(format!(
                    "Invalid UTF-8 sequence: {}",
                    e
                )))
            }
        };

        let stderr = match String::from_utf8(output.stderr) {
            Ok(text) => text,
            Err(e) => {
                return Err(TerraError::CommandFailed(format!(
                    "Invalid UTF-8 sequence: {}",
                    e
                )))
            }
        };

        if !output.status.success() {
            let command_error = format!("{} {}", stdout.to_string(), stderr.to_string());
            println!("{}", command_error);
            return Err(TerraError::CommandFailed(command_error));
        }

        Ok(stdout.to_string())
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
