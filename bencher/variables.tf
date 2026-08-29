variable "aws_region" {
  type        = string
  default     = "us-east-1"
  description = "AWS region (us-east-1 has broad availability of c7i/m7i and m7g instances)"
}

variable "aws_profile" {
  type        = string
  default     = null
  description = "Optional AWS CLI profile name"
}

variable "arch" {
  type        = string
  default     = "x86_64"
  description = "Target CPU architecture ('x86_64' or 'aarch64')"
}

variable "instance_type" {
  type        = string
  default     = ""
  description = "EC2 instance type (defaults to m7i.4xlarge for x86_64 and m7g.4xlarge for aarch64 - 64 GiB DDR5 RAM, 16 vCPUs)"
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

variable "ssh_key_path" {
  type        = string
  default     = ""
  description = "Optional custom destination path for the generated private SSH key"
}
