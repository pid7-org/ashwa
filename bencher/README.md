# Ashwa AWS Automated Benchmark Runner

Automated, single-instance, multi-target AWS EC2 Spot benchmark runner for Ashwa.

## Overview

- **Ephemeral**: Provisions a single spot instance (default `c6i.2xlarge` Ice Lake with AVX-512BW support), runs all configurations sequentially, streams output, saves results, and automatically tears down the instance via Terraform.
- **CPU Isolation**: Pins benchmark processes to an isolated CPU core (`taskset -c <core>`), sets performance CPU governor, and disables ASLR for jitter-free benchmarking.
- **Comprehensive x86_64 SIMD Matrix**:
  - `SWAR`: Software Architecture 64-bit word fallback (`--cfg forced_swar_backend`)
  - `SSE2`: 128-bit SIMD (`-C target-feature=+sse2`)
  - `AVX2`: 256-bit SIMD (`-C target-feature=+avx2`)
  - `AVX-512BW`: 512-bit Byte/Word SIMD (`-C target-feature=+avx512bw`)
  - `Native`: Runtime CPUID dynamic dispatch (`-C target-cpu=native`)

## Usage

From the project root:

```bash
# Run with default profile and current git ref
./bench-aws

# Run with a specific AWS profile
./bench-aws my-profile

# Run with custom options
./bench-aws --profile production --instance c6i.4xlarge --ref master --core 2
```

### CLI Options

| Option | Description | Default |
|--------|-------------|---------|
| `profile` / `-p, --profile` | AWS CLI profile name | `default` (or `$AWS_PROFILE`) |
| `-r, --region` | AWS Region | `us-east-1` |
| `-i, --instance` | EC2 instance type (AVX-512BW ready) | `c6i.2xlarge` |
| `-b, --ref` | Git branch, tag, or commit hash | Current git HEAD |
| `-c, --core` | CPU core ID for process pinning | `2` |
| `--on-demand` | Use On-Demand pricing instead of Spot | Spot enabled (`true`) |
| `--keep-alive` | Prevent auto-destruction on exit (debug only) | Disabled (`false`) |
| `-h, --help` | Display usage instructions | |

## Architecture

1. `bench-aws`: Local bash wrapper that manages Terraform lifecycle, SSH polling, and teardown via EXIT trap.
2. `bencher/main.tf`: Terraform configuration for security group, dynamic ED25519 SSH key generation, and EC2 spot instance with user-data bootstrap.
3. `bencher/scripts/run_benchmarks.sh`: Remote execution script that runs all x86_64 target feature combinations sequentially on an isolated CPU core and builds a consolidated summary table.
