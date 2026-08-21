# Ashwa Bencher 🐎

Automated ephemeral AWS EC2 benchmark orchestrator for `ashwa`.

## Usage

```bash
# Run from bencher directory
./bench-aws

# Run with specific profile and instance
./bench-aws default --instance c6i.4xlarge --core 2
```

## Options

| Option | Description | Default |
|:-------|:------------|:--------|
| `-p, --profile <name>` | AWS profile name | `$AWS_PROFILE` or `default` |
| `-r, --region <region>` | AWS region | `us-east-1` |
| `-i, --instance <type>` | EC2 instance type | `c6i.2xlarge` |
| `-b, --ref <git-ref>` | Git ref / commit to benchmark | current `HEAD` |
| `-c, --core <id>` | CPU core to pin benchmarks | `2` |
| `--on-demand` | Use On-Demand instead of Spot | Spot (`true`) |
| `--keep-alive` | Retain instance on exit (debug only) | `false` |

