variable "aws_region" {
  type        = string
  default     = "us-east-1"
  description = "AWS region (us-east-1 has broad availability of Ice Lake c6i instances with AVX-512BW)"
}

variable "aws_profile" {
  type        = string
  default     = null
  description = "Optional AWS CLI profile name"
}

variable "instance_type" {
  type        = string
  default     = "c6i.2xlarge"
  description = "EC2 instance type (Intel Xeon Ice Lake with AVX-512BW / AVX2 / SSE4.2 / SSSE3 / SSE2)"
}

variable "use_spot" {
  type        = bool
  default     = true
  description = "Whether to use Spot instance pricing for cost efficiency"
}

variable "root_volume_size" {
  type        = number
  default     = 30
  description = "Size of the root EBS volume in GB"
}

variable "git_repo" {
  type        = string
  default     = "https://github.com/pid7-org/ashwa.git"
  description = "Git repository URL to clone"
}

variable "git_ref" {
  type        = string
  default     = "master"
  description = "Git branch, tag, or commit hash to benchmark"
}

variable "cpu_core" {
  type        = number
  default     = 2
  description = "CPU core ID to pin benchmarks to using taskset"
}
