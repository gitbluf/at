# Any Terra (AT)

Any Terra (AT) is a simple CLI tool that ensures you always run the correct version of Terraform, and soon OpenTofu, based on the `required_version` specified in your Terraform configuration file. 
This tool is inspired by [anyzig](https://github.com/marler8997/anyzig).

## Features
- Automatically detects the required Terraform version from the `required_version` field in `terraform.tf` or `terraform.hcl` files.
- Downloads and caches the correct Terraform version if it's not already available locally.
- Works with conventional Terraform version constraints to resolve the appropriate executable.

## TODO
- Introduction of support for OpenTofu with similar functionality, ensuring compatibility and ease of use across different infrastructure management tools.

## Installation
To install Any Terra, visit [relases](https://github.com/github/at) and choose the download link or installation instructions that correspond to your operating system and architecture.
Alternatively, you can manually download and extract the appropriate archive from our [Releases page](https://github.com/gitbluf/at/releases).

## Example Usage
```bash
# Automatically determines and runs the required Terraform version for the current project
at plan [args]
```
