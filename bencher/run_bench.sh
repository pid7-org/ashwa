#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [ ! -f "id_ed25519" ]; then
    echo "Error: id_ed25519 key not found. Run 'terraform apply' first to create the instance."
    exit 1
fi

PUBLIC_IP=$(terraform output -raw public_ip 2>/dev/null || true)
if [ -z "$PUBLIC_IP" ]; then
    echo "Error: Could not retrieve public IP from terraform outputs. Is the instance running?"
    exit 1
fi

chmod 600 id_ed25519

echo "Connecting to $PUBLIC_IP to run benchmarks..."
echo "--------------------------------------------------"

# Pass any extra flags passed to this script into CRITERION_ARGS
EXTRA_ARGS="$*"

ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i id_ed25519 "ubuntu@$PUBLIC_IP" \
    "CRITERION_ARGS='$EXTRA_ARGS' /home/ubuntu/run_benchmarks.sh"

echo "--------------------------------------------------"
mkdir -p results
scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -i id_ed25519 \
    "ubuntu@$PUBLIC_IP:/home/ubuntu/results/benchmark_results.log" results/benchmark_results.log || true

echo "Results saved to bencher/results/benchmark_results.log"
