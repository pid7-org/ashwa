#!/bin/bash
set -euo pipefail

CPU_CORE="${CPU_CORE:-2}"
RESULTS_DIR="${HOME}/results"
RESULTS_FILE="${RESULTS_DIR}/benchmark_results.log"
CRITERION_ARGS="${CRITERION_ARGS:-}"

mkdir -p "$RESULTS_DIR"

# Ensure cargo is in PATH
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

cd "$HOME/ashwa"

run_all() {
    echo "================================================================="
    echo "                  SYSTEM & CPU INFORMATION                       "
    echo "================================================================="
    echo "Architecture: $(uname -m)"
    echo "Kernel:       $(uname -r)"
    echo "CPUs:         $(nproc)"
    echo "Memory:       $(free -h | awk '/^Mem:/ {print $2}')"
    echo "Pinned Core:  CPU $CPU_CORE (via taskset -c $CPU_CORE)"
    echo ""
    echo "CPU Model & Core Details:"
    lscpu | grep -E "Model name|Flags|Architecture|CPU\(s\):|Thread|Core" || true
    echo ""
    echo "Checking target features:"
    for flag in sse2 ssse3 sse4_2 avx2 avx512f avx512bw avx512vl avx512cd avx512dq avx512vbmi avx512vbmi2; do
        if grep -q "\b$flag\b" /proc/cpuinfo; then
            echo "  [x] $flag is SUPPORTED"
        else
            echo "  [ ] $flag is NOT supported"
        fi
    done
    echo "================================================================="
    echo ""
    echo "Rust Toolchains:"
    rustc +stable --version
    rustc +nightly --version || true
    cargo --version
    echo ""

    # 1. SWAR Benchmark (stable)
    echo "================================================================="
    echo " [1/4] Running SWAR Benchmark (forced_swar_backend) on CPU $CPU_CORE"
    echo "================================================================="
    RUSTFLAGS="--cfg forced_swar_backend" taskset -c "$CPU_CORE" cargo bench -p ashwa --bench one_throughput -- --nocapture $CRITERION_ARGS
    echo ""

    # 2. SSE2 Benchmark (stable)
    echo "================================================================="
    echo " [2/4] Running SSE2 Benchmark (-C target-feature=+sse2) on CPU $CPU_CORE"
    echo "================================================================="
    RUSTFLAGS="-C target-feature=+sse2" taskset -c "$CPU_CORE" cargo bench -p ashwa --bench one_throughput -- --nocapture $CRITERION_ARGS
    echo ""

    # 3. AVX2 Benchmark (stable)
    echo "================================================================="
    echo " [3/4] Running AVX2 Benchmark (-C target-feature=+avx2) on CPU $CPU_CORE"
    echo "================================================================="
    RUSTFLAGS="-C target-feature=+avx2" taskset -c "$CPU_CORE" cargo bench -p ashwa --bench one_throughput -- --nocapture $CRITERION_ARGS
    echo ""

    # 4. AVX-512BW Benchmark (nightly)
    echo "================================================================="
    echo " [4/4] Running AVX512BW Benchmark (nightly, -C target-feature=+avx512bw) on CPU $CPU_CORE"
    echo "================================================================="
    RUSTFLAGS="-C target-feature=+avx512bw" taskset -c "$CPU_CORE" cargo +nightly bench -p ashwa --bench one_throughput -- --nocapture $CRITERION_ARGS
    echo ""

    echo "================================================================="
    echo "               ALL BENCHMARKS COMPLETED SUCCESSFULLY             "
    echo "================================================================="
}

# Run and simultaneously save to file
run_all 2>&1 | tee "$RESULTS_FILE"

echo ""
echo "Results successfully saved to: $RESULTS_FILE"
