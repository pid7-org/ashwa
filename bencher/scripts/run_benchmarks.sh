#!/bin/bash
set -euo pipefail

CPU_CORE="${CPU_CORE:-2}"
RESULTS_DIR="${HOME}/results"
mkdir -p "$RESULTS_DIR"
FULL_LOG="${RESULTS_DIR}/benchmark_full.log"
SUMMARY_FILE="${RESULTS_DIR}/summary.txt"

# Ensure cargo is in PATH
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
fi

cd "$HOME/ashwa"

# Ensure system is tuned for consistent benchmarking
sudo sysctl -w kernel.randomize_va_space=0 >/dev/null 2>&1 || true
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor >/dev/null 2>&1 || true

echo "================================================================================"
echo "                   ASHWA AWS EC2 BENCHMARK RUNNER                               "
echo "================================================================================"
echo "Date:         $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "Host:         $(hostname)"
echo "Architecture: $(uname -m)"
echo "Kernel:       $(uname -r)"
echo "CPUs Total:   $(nproc)"
echo "Memory:       $(free -h | awk '/^Mem:/ {print $2}')"
echo "Pinned Core:  CPU $CPU_CORE (via taskset -c $CPU_CORE)"
echo ""
echo "CPU Model & Specs:"
lscpu | grep -E "Model name|Flags|Architecture|CPU\(s\):|Thread|Core" || true
echo ""
echo "x86_64 Vector Extension Capabilities:"
for flag in sse2 ssse3 sse4_2 avx2 avx512f avx512bw avx512vl avx512cd avx512dq avx512vbmi avx512vbmi2; do
    if grep -q "\b$flag\b" /proc/cpuinfo; then
        echo "  [x] $flag: SUPPORTED"
    else
        echo "  [ ] $flag: NOT supported on this CPU"
    fi
done
echo ""
echo "Rust Toolchain Information:"
rustc --version
cargo --version
echo "Git Commit / Ref: $(git rev-parse --short HEAD) ($(git branch --show-current 2>/dev/null || echo 'detached'))"
echo "================================================================================"
echo ""

# Declare array of configurations: [Name] [RUSTFLAGS] [LOG_FILENAME]
CONFIGS=(
    "SWAR (Forced SWAR Backend)|--cfg forced_swar_backend|swar.log"
    "SSE2 (128-bit SIMD)|-C target-feature=+sse2|sse2.log"
    "SSSE3 (Supplemental SSE3)|-C target-feature=+ssse3|ssse3.log"
    "SSE4.2 (SSE 4.2)|-C target-feature=+sse4.2|sse4_2.log"
    "AVX2 (256-bit SIMD)|-C target-feature=+avx2|avx2.log"
    "AVX-512BW (512-bit SIMD)|-C target-feature=+avx512bw|avx512bw.log"
    "Native (CPUID Auto-Dispatch)|-C target-cpu=native|native.log"
)

declare -A RESULTS_MAP

total_configs=${#CONFIGS[@]}
idx=1

for entry in "${CONFIGS[@]}"; do
    IFS='|' read -r name flags log_file <<< "$entry"
    
    echo "================================================================================"
    echo " [$idx/$total_configs] BENCHMARK: $name"
    echo " RUSTFLAGS: \"$flags\""
    echo " CPU Pinning: Core $CPU_CORE"
    echo "================================================================================"
    
    # Flush disk/system caches before starting run
    sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true
    
    # Run benchmark pinned to the selected isolated CPU core
    set +e
    RUSTFLAGS="$flags" taskset -c "$CPU_CORE" cargo bench -p ashwa --bench one_throughput -- --nocapture 2>&1 | tee "${RESULTS_DIR}/${log_file}"
    status=$?
    set -e
    
    if [ $status -ne 0 ]; then
        echo "[!] Error: Benchmark failed for configuration: $name (exit code: $status)"
        RESULTS_MAP["$name"]="FAILED"
    else
        # Extract throughput summary line if present
        thrpt=$(grep -E "^Throughput" "${RESULTS_DIR}/${log_file}" | tail -n 1 || true)
        if [ -n "$thrpt" ]; then
            RESULTS_MAP["$name"]="$thrpt"
        else
            RESULTS_MAP["$name"]="COMPLETED"
        fi
    fi
    
    echo ""
    idx=$((idx + 1))
done

# Generate Summary Report
echo "================================================================================"
echo "                   CONSOLIDATED BENCHMARK SUMMARY                               "
echo "================================================================================"
printf "| %-32s | %-40s |\n" "Configuration" "Result / Throughput"
printf "|----------------------------------|------------------------------------------|\n"

for entry in "${CONFIGS[@]}"; do
    IFS='|' read -r name flags log_file <<< "$entry"
    res="${RESULTS_MAP[$name]:-N/A}"
    printf "| %-32s | %-40s |\n" "$name" "$res"
done
echo "================================================================================"

# Write summary to disk
{
    echo "ASHWA BENCHMARK SUMMARY ($(date -u '+%Y-%m-%d %H:%M:%S UTC'))"
    echo "CPU: $(lscpu | grep 'Model name' | sed 's/Model name:[ \t]*//')"
    echo "Commit: $(git rev-parse HEAD)"
    echo "--------------------------------------------------------------------------------"
    for entry in "${CONFIGS[@]}"; do
        IFS='|' read -r name flags log_file <<< "$entry"
        res="${RESULTS_MAP[$name]:-N/A}"
        printf "%-35s : %s\n" "$name" "$res"
    done
} > "$SUMMARY_FILE"

echo ""
echo "Complete logs saved in: $RESULTS_DIR"
