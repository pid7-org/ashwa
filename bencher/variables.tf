variable "aws_region" {
  type        = string
  default     = "us-east-1"
  description = "AWS region (us-east-1 supports d3en.4xlarge)"
}

variable "instance_type" {
  type        = string
  default     = "d3en.4xlarge"
  description = "EC2 instance type (16 vCPU, 64 GiB RAM, Cascade Lake with AVX-512BW)"
}

variable "use_spot" {
  type        = bool
  default     = true
  description = "Whether to use Spot instance pricing"
}

variable "rust_version" {
  type        = string
  default     = "stable"
  description = "Rust toolchain version to install"
}

variable "root_volume_size" {
  type        = number
  default     = 50
  description = "Size of the root EBS volume in GB"
}
