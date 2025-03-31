use hcl::de::from_str;
use regex::Regex;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct TerraformBlock {
    required_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
    terraform: Option<TerraformBlock>,
}

pub fn find_required_version() -> Result<String, Box<dyn Error>> {
    let dir_path = ".";

    let mut found_version: Option<String> = None;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path: PathBuf = entry.path();

        if path
            .extension()
            .map(|s| s == "hcl" || s == "tf")
            .unwrap_or(false)
        {
            match fs::read_to_string(&path) {
                Ok(content) => match from_str::<Config>(&content) {
                    Ok(config) => {
                        if let Some(terraform) = config.terraform {
                            if let Some(required_version) = terraform.required_version {
                                if found_version.is_none() {
                                    found_version = Some(
                                        terraform_required(&required_version)
                                            .unwrap_or_else(|| "Version not found..".to_string()),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read {}: {}", path.display(), e);
                    continue;
                }
            }
        }
    }
    // Return the first found version or an error if none found
    Ok(found_version.ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No Terraform version found in any files",
        ))
    })?)
}

fn terraform_required(content: &str) -> Option<String> {
    let version_regex = Regex::new(r"[~=!<>]{0,2}\s*([0-9]+\.[0-9]+(?:\.[0-9]+)?)").unwrap();
    let three_part_version_regex = Regex::new(r"[0-9]+\.[0-9]+\.[0-9]+").unwrap();

    if let Some(captures) = version_regex.captures(content) {
        let mut version = captures.get(1).unwrap().as_str().to_string();
        // Add .0 to versions until we have three parts
        // FIX: try to find the latest patch if not specified
        while !three_part_version_regex.is_match(&version) {
            version.push_str(".0");
        }
        Some(version)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terraform_required() {
        assert_eq!(Some("1.2.0".to_string()), terraform_required(">= 1.2"));
        assert_eq!(Some("1.2.3".to_string()), terraform_required("~> 1.2.3"));
        assert_eq!(Some("1.2.4".to_string()), terraform_required("<= 1.2.4"));
        assert_eq!(None, terraform_required("no version here"));
    }
}
