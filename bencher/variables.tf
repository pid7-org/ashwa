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

variable "root_volume_size" {
  type        = number
  default     = 50
  description = "Size of the root EBS volume in GB"
}

variable "git_repo" {
  type        = string
  default     = "https://github.com/pid7-org/ashwa.git"
  description = "Git repository URL to clone"
}

variable "git_branch" {
  type        = string
  default     = "master"
  description = "Git branch to benchmark"
}

variable "cpu_core" {
  type        = number
  default     = 2
  description = "CPU core ID to pin benchmarks to using taskset"
}

variable "criterion_args" {
  type        = string
  default     = ""
  description = "Optional additional flags for Criterion (e.g. '--warm-up-time 1 --measurement-time 2')"
}
