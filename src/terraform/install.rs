use dirs::home_dir;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod install {
    use super::*;
    // Installs a specific version of Terraform CLI to the local cache
    // Returns terraform binary path
    pub fn install(version: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create cache directory if it doesn't exist
        let cache_path = PathBuf::from(get_cache_path());
        fs::create_dir_all(&cache_path)?;

        let version_path = cache_path.join(version);
        let terraform_path = version_path.join("terraform");

        if terraform_path.exists() {
            println!("Version {} already exists", version);
            return Ok(terraform_path.display().to_string());
        }
        println!("Installing Terraform version {}", version);

        // example: terraform_1.9.0_darwin_arm64.zip
        let url = format!(
            "https://releases.hashicorp.com/terraform/{}/terraform_{}_{}.zip",
            version,
            version,
            get_platform_string()?
        );

        let temp_dir = tempfile::tempdir()?;
        let zip_path = temp_dir.path().join("terraform.zip");

        println!("Downloading from: {}", url);
        download_file(&url, &zip_path)?;

        // Extract zip file
        println!("Extracting Terraform binary...");
        let mut zip = zip::ZipArchive::new(fs::File::open(&zip_path)?)?;
        zip.extract(temp_dir.path())?;

        // Move binary to cache directory
        let temp_terraform = temp_dir.path().join("terraform");
        fs::create_dir_all(&version_path)?;
        fs::copy(&temp_terraform, &terraform_path)?;

        // Make binary executable
        fs::set_permissions(&terraform_path, fs::Permissions::from_mode(0o755))?;

        // Verify installation
        println!("Verifying installation...");
        verify_installation(&terraform_path)?;

        Ok(terraform_path.display().to_string())
    }

    fn verify_installation(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new(path).arg("version").output()?;

        if !output.status.success() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            )));
        }

        println!("{}", String::from_utf8_lossy(&output.stdout));
        Ok(())
    }

    fn download_file(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut response = reqwest::blocking::get(url)?;
        let mut file = fs::File::create(path)?;
        io::copy(&mut response, &mut file)?;
        Ok(())
    }

    fn get_cache_path() -> String {
        match home_dir() {
            Some(home) => {
                let mut path = PathBuf::from(home);
                path.extend([".cache", "at", "terraform"]);
                path.into_os_string()
                    .into_string()
                    .expect("Path contains invalid UTF-8")
            }
            None => String::from(format!("/tmp/.cache/at/terraform")), // Fallback path
        }
    }

    // Returns the platform-specific string for Terraform downloads
    #[allow(unexpected_cfgs)]
    fn get_platform_string() -> Result<String, Box<dyn std::error::Error>> {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else {
            "linux"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "amd64"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Unsupported architecture: {}", cfg!(target_arch)),
            )));
        };

        Ok(format!("{}_{}", os, arch))
    }
}
