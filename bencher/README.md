# Ashwa Bencher 🐎

Automated ephemeral AWS EC2 benchmark orchestrator for `ashwa` across **x86_64** (Intel AVX-512BW / AVX2)
and **aarch64** (AWS Graviton ARM NEON) architectures.

## AWS Authentication

Configure AWS credentials with permissions for EC2 and VPC resources:

```sh
aws configure
```

## Usage

```bash
./bench-aws --target two --aarch64 --ref master --core 0
```

## Options

| Option                  | Description                                                                     | Default                       |
|:------------------------|:--------------------------------------------------------------------------------|:------------------------------|
| `-t, --target <t>`      | Benchmark target: `one` (`search_one`), `two` (`search_two`), or `three` (`search_three`) | *Required*                    |
| `-a, --arch <arch>`     | Target architecture: `x86_64`, `aarch64`, or `all`/`parallel`                   | `x86_64`                      |
| `-p, --profile <name>`  | AWS CLI profile name                                                            | `$AWS_PROFILE` / `default`    |
| `-r, --region <region>` | AWS Region                                                                      | `us-east-1`                   |
| `-i, --instance <type>` | Override EC2 instance type (16 vCPUs, 64 GiB DDR5 RAM)                          | `m7i.4xlarge` / `m7g.4xlarge` |
| `-b, --ref <git-ref>`   | Git branch, tag, or commit hash to benchmark                                    | current `HEAD`                |
| `-c, --core <id>`       | CPU core to pin benchmarks (`taskset`)                                          | `2`                           |
| `--on-demand`           | Use On-Demand instances instead of Spot                                         | Spot (`true`)                 |
| `--keep-alive`          | Retain instance on exit (debug only)                                            | `false`                       |

## Results Structure

Artifacts are isolated under `bencher/results/` per target:

- `results/one/`: Single-byte search reports & logs (`x86_64/`, `aarch64/`).
- `results/two/`: Two-byte search reports & logs (`x86_64/`, `aarch64/`).
- `results/three/`: Three-byte search reports & logs (`x86_64/`, `aarch64/`).
