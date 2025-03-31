variable "example_variable" {
  description = "An example variable"
  type        = string
  default     = "Hello, World!"
}

output "example_output" {
  description = "Outputs the value of the example variable"
  value       = var.example_variable
}
