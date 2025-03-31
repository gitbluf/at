mod terraform;

use std::error::Error;
use terraform::install::install;
use terraform::run;
use terraform::version::find_required_version;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match find_required_version() {
        Ok(version) => {
            println!("Found version: {}", version);
            match install::install(&version) {
                Ok(path) => {
                    if !args.is_empty() {
                        println!("Running terraform with arguments: {:?}", args);
                        let terraform =
                            run::TerraExecutor::new(path).expect("Failed to create executor");
                        terraform.execute_command(&args)?;
                    }
                }
                Err(e) => {
                    eprintln!("Installation failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("Error finding version: {}", e);
            return Err(e.into());
        }
    }
    Ok(())
}
