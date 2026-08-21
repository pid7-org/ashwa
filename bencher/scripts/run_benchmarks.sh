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

# Allow perf profiling without root restrictions
sudo sysctl -w kernel.perf_event_paranoid=-1 >/dev/null 2>&1 || true
sudo sysctl -w kernel.kptr_restrict=0 >/dev/null 2>&1 || true

echo "================================================================================"
echo "                   ASHWA AVX-512BW BENCHMARK & ILP RUNNER                       "
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
for flag in sse2 avx2 avx512f avx512bw avx512vl avx512cd avx512dq avx512vbmi avx512vbmi2; do
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

RUSTFLAGS="-C target-feature=+avx512bw"
export RUSTFLAGS

# ==============================================================================
# SECTION 1: LATENCY & THROUGHPUT BENCHMARK (4 CACHE/RAM TIERS)
# ==============================================================================
echo "================================================================================"
echo " [1/2] RUNNING AVX-512BW THROUGHPUT & LATENCY BENCHMARK"
echo " Target Feature: avx512bw"
echo " Payload Tiers:  L1 Cache (32 KiB), L2 Cache (512 KiB), L3 Cache (16 MiB), RAM (256 MiB)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "================================================================================"

# Flush system cache
sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true

taskset -c "$CPU_CORE" cargo bench -p ashwa --bench one_throughput -- --nocapture 2>&1 | tee "${RESULTS_DIR}/throughput_latency.log"
echo ""

# ==============================================================================
# SECTION 2: INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & HARDWARE PROFILING
# ==============================================================================
echo "================================================================================"
echo " [2/2] MEASURING INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & CPU METRICS"
echo " Harness:        core/examples/one_ilp.rs (Sample Size: 1,000 iterations per tier)"
echo " Profiler:       Linux perf stat (hardware PMU counters)"
echo " Target Feature: avx512bw"
echo " CPU Pinning:    Core $CPU_CORE"
echo "================================================================================"

echo "Compiling one_ilp release binary with AVX-512BW..."
cargo build --release -p ashwa --example one_ilp

ILP_BIN="./target/release/examples/one_ilp"
if [ ! -f "$ILP_BIN" ]; then
    ILP_BIN="./target/release/one_ilp"
fi

TIERS=("l1" "l2" "l3" "ram")
TIER_LABELS=("L1 Cache (32 KiB)" "L2 Cache (512 KiB)" "L3 Cache (16 MiB)" "Memory Bound (RAM 256 MiB)")

ILP_LOG="${RESULTS_DIR}/ilp_perf.log"
: > "$ILP_LOG"

declare -A MAP_INSN
declare -A MAP_CYCLES
declare -A MAP_IPC
declare -A MAP_GHZ
declare -A MAP_BRANCH_MISS

for idx in "${!TIERS[@]}"; do
    tier="${TIERS[$idx]}"
    label="${TIER_LABELS[$idx]}"
    
    echo "Profiling $label..."
    sync && echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true
    
    if command -v perf >/dev/null 2>&1; then
        # Run perf stat and capture CSV metrics
        PERF_OUT=$(sudo taskset -c "$CPU_CORE" perf stat -x ';' -e instructions,cycles,task-clock,branches,branch-misses "$ILP_BIN" "$tier" 2>&1 || true)
        echo "=== $label ===" >> "$ILP_LOG"
        echo "$PERF_OUT" >> "$ILP_LOG"
        echo "" >> "$ILP_LOG"
        
        # Parse perf stat CSV output
        insn=$(echo "$PERF_OUT" | awk -F';' '/instructions/ {print $1}' | tr -d ' ' || echo "0")
        cycles=$(echo "$PERF_OUT" | awk -F';' '/\<cycles\>/ {print $1}' | tr -d ' ' || echo "0")
        task_clock=$(echo "$PERF_OUT" | awk -F';' '/task-clock/ {print $1}' | tr -d ' ' || echo "0")
        b_miss=$(echo "$PERF_OUT" | awk -F';' '/branch-misses/ {print $1}' | tr -d ' ' || echo "0")
        b_total=$(echo "$PERF_OUT" | awk -F';' '/\<branches\>/ {print $1}' | tr -d ' ' || echo "0")
        
        if [ -n "$insn" ] && [ -n "$cycles" ] && [ "$cycles" -gt 0 ] 2>/dev/null; then
            ipc=$(awk -v i="$insn" -v c="$cycles" 'BEGIN { printf "%.2f", i/c }')
        else
            ipc="N/A"
        fi
        
        if [ -n "$cycles" ] && [ -n "$task_clock" ] && (( $(echo "$task_clock > 0" | bc -l 2>/dev/null || echo 0) )); then
            ghz=$(awk -v c="$cycles" -v t="$task_clock" 'BEGIN { printf "%.2f GHz", (c / (t * 1000000)) }')
        else
            ghz="N/A"
        fi
        
        if [ -n "$b_miss" ] && [ -n "$b_total" ] && [ "$b_total" -gt 0 ] 2>/dev/null; then
            b_miss_pct=$(awk -v m="$b_miss" -v tot="$b_total" 'BEGIN { printf "%.2f%%", (m/tot)*100 }')
        else
            b_miss_pct="0.00%"
        fi
        
        MAP_INSN["$tier"]="$insn"
        MAP_CYCLES["$tier"]="$cycles"
        MAP_IPC["$tier"]="$ipc"
        MAP_GHZ["$tier"]="$ghz"
        MAP_BRANCH_MISS["$tier"]="$b_miss_pct"
    else
        echo "Warning: perf tool not found. Skipping hardware PMU metrics."
        MAP_IPC["$tier"]="N/A"
        MAP_GHZ["$tier"]="N/A"
        MAP_BRANCH_MISS["$tier"]="N/A"
    fi
done

echo ""
echo "================================================================================"
echo "                 AVX-512BW HARDWARE PERFORMANCE & ILP METRICS                   "
echo "================================================================================"
printf "| %-28s | %-16s | %-16s | %-16s |\n" "Tier / Level" "ILP (IPC)" "CPU Frequency" "Branch Miss %"
printf "|------------------------------|------------------|------------------|------------------|\n"
for idx in "${!TIERS[@]}"; do
    tier="${TIERS[$idx]}"
    label="${TIER_LABELS[$idx]}"
    printf "| %-28s | %-16s | %-16s | %-16s |\n" \
        "$label" \
        "${MAP_IPC[$tier]:-N/A} insn/cyc" \
        "${MAP_GHZ[$tier]:-N/A}" \
        "${MAP_BRANCH_MISS[$tier]:-N/A}"
done
echo "================================================================================"
echo ""

# Write consolidated summary to file
{
    echo "ASHWA AVX-512BW BENCHMARK & HARDWARE PROFILING SUMMARY"
    echo "Date:   $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "CPU:    $(lscpu | grep 'Model name' | sed 's/Model name:[ \t]*//')"
    echo "Commit: $(git rev-parse HEAD)"
    echo "Pinned: CPU $CPU_CORE"
    echo "================================================================================"
    echo ""
    echo "1. THROUGHPUT & LATENCY (AVX-512BW):"
    cat "${RESULTS_DIR}/throughput_latency.log" 2>/dev/null || true
    echo ""
    echo "2. INSTRUCTION-LEVEL PARALLELISM (ILP) & HARDWARE METRICS:"
    printf "| %-28s | %-16s | %-16s | %-16s |\n" "Tier / Level" "ILP (IPC)" "CPU Frequency" "Branch Miss %"
    printf "|------------------------------|------------------|------------------|------------------|\n"
    for idx in "${!TIERS[@]}"; do
        tier="${TIERS[$idx]}"
        label="${TIER_LABELS[$idx]}"
        printf "| %-28s | %-16s | %-16s | %-16s |\n" \
            "$label" \
            "${MAP_IPC[$tier]:-N/A} insn/cyc" \
            "${MAP_GHZ[$tier]:-N/A}" \
            "${MAP_BRANCH_MISS[$tier]:-N/A}"
    done
} > "$SUMMARY_FILE"

echo "Results successfully saved to: $RESULTS_DIR"
