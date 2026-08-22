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
  region  = var.aws_region
  profile = var.aws_profile
}

locals {
  is_arm                = var.arch == "aarch64" || var.arch == "arm64"
  arch_norm             = local.is_arm ? "aarch64" : "x86_64"
  default_instance_type = local.is_arm ? "m7g.4xlarge" : "m6i.4xlarge"
  instance_type         = var.instance_type != "" ? var.instance_type : local.default_instance_type
  key_file_path         = var.ssh_key_path != "" ? var.ssh_key_path : "${path.module}/id_ed25519_${local.arch_norm}"
}

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = [local.is_arm ? "ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*" : "ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-amd64-server-*"]
  }

  filter {
    name   = "architecture"
    values = [local.is_arm ? "arm64" : "x86_64"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

data "aws_vpc" "default" {
  default = true
}

# NOTE: Ephemeral security group allowing SSH for runner communication
resource "aws_security_group" "bench" {
  name_prefix = "ashwa-bench-sg-${local.arch_norm}-"
  description = "Security group for ephemeral Ashwa benchmark instance (${local.arch_norm})"
  vpc_id      = data.aws_vpc.default.id

  ingress {
    description = "SSH runner control"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    description = "Outbound package repositories and toolchains"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "ashwa-bench-sg-${local.arch_norm}"
  }
}

# NOTE: Dynamically generated in-memory ED25519 keypair for automated session
resource "tls_private_key" "ssh" {
  algorithm = "ED25519"
}

resource "aws_key_pair" "bench" {
  key_name_prefix = "ashwa-bench-key-${local.arch_norm}-"
  public_key      = tls_private_key.ssh.public_key_openssh
}

resource "local_file" "private_key" {
  content         = tls_private_key.ssh.private_key_openssh
  filename        = local.key_file_path
  file_permission = "0600"
}

resource "aws_instance" "bench" {
  ami                         = data.aws_ami.ubuntu.id
  instance_type               = local.instance_type
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

  user_data = <<-EOF
              #!/bin/bash
              sysctl -w kernel.randomize_va_space=0 || true
              EOF

  tags = {
    Name        = "ashwa-bench-runner-${local.arch_norm}"
    Environment = "ephemeral-bench"
    Arch        = local.arch_norm
  }
}
