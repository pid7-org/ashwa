# Ashwa Bencher 🐎

Automated ephemeral AWS EC2 benchmark orchestrator for `ashwa` across **x86_64** (Intel AVX-512BW / AVX2) and **aarch64** (AWS Graviton ARM NEON) architectures with 64 GiB RAM machines (<= 16 vCPUs).

## Usage

```bash
# Run search_one on default x86_64 (m6i.4xlarge - 64 GiB RAM, 16 vCPUs)
./bench-aws --target one

# Run search_two on aarch64 (m7g.4xlarge - 64 GiB RAM, 16 vCPUs, ARM NEON)
./bench-aws --target two --aarch64

# Run search_two on BOTH x86_64 and aarch64 concurrently in parallel
./bench-aws --two --parallel

# Run with custom profile, region, branch, and core pinning
./bench-aws default --target two --arch aarch64 --region us-east-1 --branch master --core 2
```

## Options

| Option | Description | Default |
|:-------|:------------|:--------|
| `-t, --target, --suite <t>` | Benchmark suite: `one` (search_one) or `two` (search_two) | **Required (No default)** |
| `--one, -1` | Benchmark single-byte search (`search_one` / `one_throughput`) | - |
| `--two, -2` | Benchmark two-byte search (`search_two` / `two_throughput`) | - |
| `-a, --arch <arch>` | Target architecture (`x86_64`, `aarch64`, or `all`/`parallel`) | `x86_64` |
| `--aarch64`, `--arm64` | Run benchmark on `aarch64` (Graviton3 with NEON) | - |
| `--x86_64`, `--x64` | Run benchmark on `x86_64` (Ice Lake with AVX-512BW) | - |
| `--parallel`, `--all` | Run **both** x86_64 and aarch64 benchmarks in parallel | - |
| `-i, --instance <type>` | Override EC2 instance type (default: 64 GiB RAM, 16 vCPUs) | `m6i.4xlarge` (x86_64) / `m7g.4xlarge` (aarch64) |
| `-p, --profile <name>` | AWS CLI profile name | `$AWS_PROFILE` or `default` |
| `-r, --region <region>` | AWS region | `us-east-1` |
| `-b, --ref <git-ref>` | Git branch, tag, or commit to benchmark | current `HEAD` |
| `-c, --core <id>` | CPU core to pin benchmarks using `taskset` | `2` |
| `--on-demand` | Use On-Demand instance instead of Spot | Spot (`true`) |
| `--keep-alive` | Retain instance on exit (debug only) | `false` |

## Results Structure

Benchmark artifacts are partitioned by benchmark target under `bencher/results/` to prevent overwriting results across different targets:

- `results/one/`: Single-byte search benchmark artifacts (`summary.txt`, throughput logs, architecture subdirectories `x86_64/`, `aarch64/`).
- `results/two/`: Two-byte search benchmark artifacts (`summary.txt`, throughput logs, architecture subdirectories `x86_64/`, `aarch64/`).



