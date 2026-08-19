#!/bin/bash
set -euo pipefail

echo "================================================================="
echo "                  SYSTEM & CPU INFORMATION                       "
echo "================================================================="
echo "Architecture: $(uname -m)"
echo "Kernel:       $(uname -r)"
echo "CPUs:         $(nproc)"
echo "Memory:       $(free -h | awk '/^Mem:/ {print $2}')"
echo ""
echo "CPU Model & ISA Flags:"
lscpu | grep -E "Model name|Flags|Architecture|CPU\(s\):|Thread|Core" || true
echo ""
echo "Checking specific target features:"
for flag in sse2 ssse3 sse4_2 avx2 avx512f avx512bw avx512vl avx512cd avx512dq avx512vbmi avx512vbmi2; do
    if grep -q "\b$flag\b" /proc/cpuinfo; then
        echo "  [x] $flag is SUPPORTED"
    else
        echo "  [ ] $flag is NOT supported"
    fi
done
echo "================================================================="
echo ""

# Source cargo environment if not already in PATH
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

cd "$HOME/ashwa"

echo "Rust version:"
rustc --version
cargo --version
echo ""

# 1. SWAR Benchmark
echo "================================================================="
echo " [1/4] Running SWAR Benchmark (forced_swar_backend)"
echo "================================================================="
RUSTFLAGS="--cfg forced_swar_backend" cargo bench -p ashwa --bench one_throughput -- --nocapture
echo ""

# 2. SSE2 Benchmark
echo "================================================================="
echo " [2/4] Running SSE2 Benchmark (-C target-feature=+sse2)"
echo "================================================================="
RUSTFLAGS="-C target-feature=+sse2" cargo bench -p ashwa --bench one_throughput -- --nocapture
echo ""

# 3. AVX2 Benchmark
echo "================================================================="
echo " [3/4] Running AVX2 Benchmark (-C target-feature=+avx2)"
echo "================================================================="
RUSTFLAGS="-C target-feature=+avx2" cargo bench -p ashwa --bench one_throughput -- --nocapture
echo ""

# 4. AVX-512BW Benchmark
echo "================================================================="
echo " [4/4] Running AVX512BW Benchmark (-C target-feature=+avx512bw)"
echo "================================================================="
RUSTFLAGS="-C target-feature=+avx512bw" cargo bench -p ashwa --bench one_throughput -- --nocapture
echo ""

echo "================================================================="
echo "               ALL BENCHMARKS COMPLETED SUCCESSFULLY             "
echo "================================================================="
