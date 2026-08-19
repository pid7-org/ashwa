terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    tls = {
      source  = "hashicorp/tls"
      version = "~> 4.0"
    }
    local = {
      source  = "hashicorp/local"
      version = "~> 2.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# 1. Latest Ubuntu Noble 24.04 LTS AMI for x86_64
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# 2. Default VPC
data "aws_vpc" "default" {
  default = true
}

# 3. Security Group for SSH & outbound traffic
resource "aws_security_group" "bench" {
  name_prefix = "ashwa-bench-sg-"
  description = "Security group for Ashwa benchmark EC2 instance"
  vpc_id      = data.aws_vpc.default.id

  ingress {
    description = "SSH access"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "All outbound traffic"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "ashwa-bench-sg"
  }
}

# 4. Generate SSH Key Pair automatically
resource "tls_private_key" "ssh" {
  algorithm = "ED25519"
}

resource "aws_key_pair" "bench" {
  key_name_prefix = "ashwa-bench-key-"
  public_key      = tls_private_key.ssh.public_key_openssh
}

resource "local_file" "private_key" {
  content         = tls_private_key.ssh.private_key_openssh
  filename        = "${path.module}/id_ed25519"
  file_permission = "0600"
}

# 5. EC2 Instance (d3en.4xlarge: 16 vCPU, 64 GiB RAM, Cascade Lake x86_64)
resource "aws_instance" "bench" {
  ami                         = data.aws_ami.ubuntu.id
  instance_type               = var.instance_type
  key_name                    = aws_key_pair.bench.key_name
  vpc_security_group_ids      = [aws_security_group.bench.id]
  associate_public_ip_address = true

  root_block_device {
    volume_size           = var.root_volume_size
    volume_type           = "gp3"
    delete_on_termination = true
  }

  dynamic "instance_market_options" {
    for_each = var.use_spot ? [1] : []
    content {
      market_type = "spot"
    }
  }

  tags = {
    Name = "ashwa-bench-d3en"
  }

  connection {
    type        = "ssh"
    user        = "ubuntu"
    private_key = tls_private_key.ssh.private_key_openssh
    host        = self.public_ip
    timeout     = "6m"
  }

  # Wait for instance cloud-init to finish
  provisioner "remote-exec" {
    inline = [
      "echo '=== Waiting for cloud-init to finish... ==='",
      "while [ ! -f /var/lib/cloud/instance/boot-finished ]; do sleep 2; done",
      "echo '=== Instance boot complete. ==='"
    ]
  }

  # Upload benchmark runner script
  provisioner "file" {
    source      = "${path.module}/scripts/run_benchmarks.sh"
    destination = "/home/ubuntu/run_benchmarks.sh"
  }

  # Install toolchains, clone repo, and execute benchmarks with CPU core pinning & ISA flags
  provisioner "remote-exec" {
    inline = [
      "set -e",
      "echo '=== 1. Updating APT & installing required packages ==='",
      "sudo apt-get update -y",
      "sudo apt-get install -y build-essential pkg-config libssl-dev git curl util-linux",
      
      "echo '=== 2. Installing Rust toolchains (stable & nightly) ==='",
      "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable",
      "source $HOME/.cargo/env",
      "rustup toolchain install nightly",

      "echo '=== 3. Cloning repository from Git ==='",
      "rm -rf $HOME/ashwa",
      "git clone --branch ${var.git_branch} ${var.git_repo} $HOME/ashwa",

      "echo '=== 4. Starting benchmark runner on pinned CPU core (${var.cpu_core}) ==='",
      "chmod +x $HOME/run_benchmarks.sh",
      "CPU_CORE='${var.cpu_core}' CRITERION_ARGS='${var.criterion_args}' $HOME/run_benchmarks.sh"
    ]
  }

  # Download the benchmark results log locally
  provisioner "local-exec" {
    command = <<-EOT
      mkdir -p ${path.module}/results
      scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i ${local_file.private_key.filename} ubuntu@${self.public_ip}:/home/ubuntu/results/benchmark_results.log ${path.module}/results/benchmark_results.log || true
      echo "Benchmark results downloaded to ${path.module}/results/benchmark_results.log"
    EOT
  }

  depends_on = [
    local_file.private_key
  ]
}
