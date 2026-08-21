#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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

# System and OS tuning for consistent benchmarking (non-interactive sudo)
sudo -n sysctl -w kernel.randomize_va_space=0 >/dev/null 2>&1 || true
echo performance | sudo -n tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor >/dev/null 2>&1 || true
sudo -n sysctl -w kernel.perf_event_paranoid=-1 >/dev/null 2>&1 || true
sudo -n sysctl -w kernel.kptr_restrict=0 >/dev/null 2>&1 || true

# Detect hostname safely
HOST_NAME=$(cat /etc/hostname 2>/dev/null || uname -n 2>/dev/null || echo "localhost")

echo "================================================================================"
echo "                   ASHWA AVX-512BW BENCHMARK & HARDWARE REPORT                  "
echo "================================================================================"
echo "Date:         $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "Host:         $HOST_NAME"
echo "Pinned Core:  CPU $CPU_CORE (via taskset -c $CPU_CORE)"
echo "================================================================================"
echo ""

# ==============================================================================
# CONTEXT 1: SYSTEM, CACHE & MEMORY TOPOLOGY PROFILING
# ==============================================================================
echo "[1/3] Detecting hardware capabilities, cache topology & STREAM memory rate..."

# 1. CPU Model
CPU_MODEL=$(lscpu | grep -E "Model name" | sed -E 's/Model name:[ \t]*//' | head -n 1 || echo "Unknown CPU")

# 2. Highest Available ISA Detection
detect_highest_isa() {
    if grep -q "\bavx512bw\b" /proc/cpuinfo 2>/dev/null && grep -q "\bavx512f\b" /proc/cpuinfo 2>/dev/null; then
        echo "AVX-512BW (512-bit SIMD)"
    elif grep -q "\bavx2\b" /proc/cpuinfo 2>/dev/null; then
        echo "AVX2 (256-bit SIMD)"
    elif grep -q "\bsse4_2\b" /proc/cpuinfo 2>/dev/null; then
        echo "SSE4.2 (128-bit SIMD)"
    elif grep -q "\bssse3\b" /proc/cpuinfo 2>/dev/null; then
        echo "SSSE3 (128-bit SIMD)"
    elif grep -q "\bsse2\b" /proc/cpuinfo 2>/dev/null; then
        echo "SSE2 (128-bit SIMD)"
    elif grep -q "\bneon\b" /proc/cpuinfo 2>/dev/null; then
        echo "ARM NEON (128-bit SIMD)"
    else
        echo "SWAR (64-bit Scalar Fallback)"
    fi
}
HIGHEST_ISA=$(detect_highest_isa)

# 3. Cache Sizes
L1D_CACHE=$(lscpu | grep -E "L1d cache:" | sed -E 's/L1d cache:[ \t]*//' | head -n 1 || echo "N/A")
L1I_CACHE=$(lscpu | grep -E "L1i cache:" | sed -E 's/L1i cache:[ \t]*//' | head -n 1 || echo "N/A")
L2_CACHE=$(lscpu | grep -E "L2 cache:" | sed -E 's/L2 cache:[ \t]*//' | head -n 1 || echo "N/A")
L3_CACHE=$(lscpu | grep -E "L3 cache:" | sed -E 's/L3 cache:[ \t]*//' | head -n 1 || echo "N/A")

# 4. STREAM Memory Benchmark (Triad Best Rate & Time)
STREAM_SRC=""
for path in "${SCRIPT_DIR}/stream.c" "${HOME}/stream.c" "${HOME}/scripts/stream.c" "${HOME}/ashwa/bencher/scripts/stream.c"; do
    if [ -f "$path" ]; then
        STREAM_SRC="$path"
        break
    fi
done

STREAM_BIN="/tmp/ashwa_stream"
STREAM_TRIAD_RATE="N/A"
STREAM_TRIAD_TIME="N/A"

if [ -n "$STREAM_SRC" ] && [ -f "$STREAM_SRC" ]; then
    gcc -O3 "$STREAM_SRC" -o "$STREAM_BIN" 2>/dev/null || true
    if [ -f "$STREAM_BIN" ]; then
        STREAM_OUT=$(taskset -c "$CPU_CORE" "$STREAM_BIN" 2>/dev/null || true)
        raw_rate=$(echo "$STREAM_OUT" | awk '/TRIAD_BEST_RATE_MB_S:/ {print $2}' || true)
        raw_time=$(echo "$STREAM_OUT" | awk '/TRIAD_MIN_TIME_S:/ {print $2}' || true)
        
        if [ -n "$raw_rate" ]; then
            rate_gb=$(awk -v r="$raw_rate" 'BEGIN { r_num = r + 0; if (r_num > 0) printf "%.2f GB/s", r_num/1024; else print "" }')
            if [ -n "$rate_gb" ]; then
                STREAM_TRIAD_RATE="$raw_rate MB/s ($rate_gb)"
            fi
        fi
        if [ -n "$raw_time" ]; then
            STREAM_TRIAD_TIME=$(awk -v t="$raw_time" 'BEGIN { t_num = t + 0; if (t_num > 0) printf "%.2f ms", t_num * 1000; else print "N/A" }')
        fi
    fi
fi

# Print Context 1 Table
echo ""
echo "================================================================================"
echo "               CONTEXT 1: SYSTEM, CACHE & MEMORY TOPOLOGY                       "
echo "================================================================================"
printf "+-%-32s-+-%-42s-+\n" "--------------------------------" "------------------------------------------"
printf "| %-32s | %-42s |\n" "Component / Metric" "Specification / Value"
printf "+-%-32s-+-%-42s-+\n" "--------------------------------" "------------------------------------------"
printf "| %-32s | %-42s |\n" "CPU Model" "$CPU_MODEL"
printf "| %-32s | %-42s |\n" "Highest Available ISA" "$HIGHEST_ISA"
printf "| %-32s | %-42s |\n" "L1 Data Cache (L1d)" "$L1D_CACHE"
printf "| %-32s | %-42s |\n" "L1 Instruction Cache (L1i)" "$L1I_CACHE"
printf "| %-32s | %-42s |\n" "L2 Cache" "$L2_CACHE"
printf "| %-32s | %-42s |\n" "L3 Cache" "$L3_CACHE"
printf "| %-32s | %-42s |\n" "STREAM Triad Best Rate" "$STREAM_TRIAD_RATE"
printf "| %-32s | %-42s |\n" "STREAM Triad Best Time" "$STREAM_TRIAD_TIME"
printf "+-%-32s-+-%-42s-+\n" "--------------------------------" "------------------------------------------"
echo ""

# ==============================================================================
# CONTEXT 2: AVX-512BW THROUGHPUT & LATENCY (4 CACHE/RAM TIERS)
# ==============================================================================
# Configure RUSTFLAGS based on hardware support (AVX-512BW on EC2 / native fallback)
if grep -q "\bavx512bw\b" /proc/cpuinfo 2>/dev/null; then
    TARGET_FLAG="-C target-feature=+avx512bw"
    TARGET_LABEL="avx512bw"
else
    TARGET_FLAG="-C target-cpu=native"
    TARGET_LABEL="native ($HIGHEST_ISA)"
fi

RUSTFLAGS="${RUSTFLAGS:-$TARGET_FLAG}"
export RUSTFLAGS

echo "================================================================================"
echo " [2/3] RUNNING THROUGHPUT & LATENCY BENCHMARK"
echo " Target Feature: $TARGET_LABEL ($RUSTFLAGS)"
echo " Payload Tiers:  L1 (32 KiB), L2 (512 KiB), L3 (16 MiB), RAM (256 MiB)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "================================================================================"

sync && echo 3 | sudo -n tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true
taskset -c "$CPU_CORE" cargo bench -p ashwa --bench one_throughput -- --nocapture 2>&1 | tee "${RESULTS_DIR}/throughput_latency.log"
echo ""

# ==============================================================================
# CONTEXT 3: INSTRUCTION-LEVEL PARALLELISM (ILP) & HARDWARE PROFILING
# ==============================================================================
echo "================================================================================"
echo " [3/3] MEASURING INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & CPU METRICS"
echo " Harness:        core/examples/one_ilp.rs (Sample Size: 1,000 iterations per tier)"
echo " Profiler:       Linux perf stat (hardware PMU counters)"
echo " Target Feature: avx512bw"
echo " CPU Pinning:    Core $CPU_CORE"
echo "================================================================================"

cargo build --release -p ashwa --example one_ilp
ILP_BIN="./target/release/examples/one_ilp"
if [ ! -f "$ILP_BIN" ]; then
    ILP_BIN="./target/release/one_ilp"
fi

TIERS=("l1" "l2" "l3" "ram")
TIER_LABELS=("L1 Cache (32 KiB)" "L2 Cache (512 KiB)" "L3 Cache (16 MiB)" "RAM (256 MiB)")

declare -A MAP_IPC
declare -A MAP_GHZ
declare -A MAP_BRANCH_MISS

for idx in "${!TIERS[@]}"; do
    tier="${TIERS[$idx]}"
    label="${TIER_LABELS[$idx]}"
    
    sync && echo 3 | sudo -n tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true
    
    if command -v perf >/dev/null 2>&1; then
        PERF_OUT=$(taskset -c "$CPU_CORE" perf stat -x ';' -e instructions,cycles,task-clock,branches,branch-misses "$ILP_BIN" "$tier" 2>&1 || sudo -n taskset -c "$CPU_CORE" perf stat -x ';' -e instructions,cycles,task-clock,branches,branch-misses "$ILP_BIN" "$tier" 2>&1 || true)
        
        insn=$(echo "$PERF_OUT" | awk -F';' '/instructions/ {print $1}' | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        cycles=$(echo "$PERF_OUT" | awk -F';' '/\<cycles\>/ {print $1}' | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        task_clock=$(echo "$PERF_OUT" | awk -F';' '/task-clock/ {print $1}' | tr -d ' ' | grep -o -E '^[0-9]+(\.[0-9]+)?' || echo "")
        b_miss=$(echo "$PERF_OUT" | awk -F';' '/branch-misses/ {print $1}' | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        b_total=$(echo "$PERF_OUT" | awk -F';' '/\<branches\>/ {print $1}' | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        
        if [ -n "$insn" ] && [ -n "$cycles" ]; then
            ipc=$(awk -v i="$insn" -v c="$cycles" 'BEGIN { c_num = c + 0; i_num = i + 0; if (c_num > 0) printf "%.2f insn/cyc", i_num / c_num; else print "N/A" }')
        else
            ipc="N/A (PMU counter not available)"
        fi
        
        if [ -n "$cycles" ] && [ -n "$task_clock" ]; then
            ghz=$(awk -v c="$cycles" -v t="$task_clock" 'BEGIN { c_num = c + 0; t_num = t + 0; if (t_num > 0) printf "%.2f GHz", (c_num / (t_num * 1000000)); else print "N/A" }')
        else
            ghz="N/A"
        fi
        
        if [ -n "$b_miss" ] && [ -n "$b_total" ]; then
            b_miss_pct=$(awk -v m="$b_miss" -v tot="$b_total" 'BEGIN { m_num = m + 0; tot_num = tot + 0; if (tot_num > 0) printf "%.3f%%", (m_num / tot_num) * 100; else print "0.000%" }')
        else
            b_miss_pct="0.000%"
        fi
        
        MAP_IPC["$tier"]="$ipc"
        MAP_GHZ["$tier"]="$ghz"
        MAP_BRANCH_MISS["$tier"]="$b_miss_pct"
    else
        MAP_IPC["$tier"]="N/A"
        MAP_GHZ["$tier"]="N/A"
        MAP_BRANCH_MISS["$tier"]="N/A"
    fi
done

echo ""
echo "================================================================================"
echo "          CONTEXT 3: INSTRUCTION-LEVEL PARALLELISM & HARDWARE METRICS           "
echo "================================================================================"
printf "+-%-24s-+-%-18s-+-%-18s-+-%-16s-+\n" "------------------------" "------------------" "------------------" "----------------"
printf "| %-24s | %-18s | %-18s | %-16s |\n" "Tier / Level" "ILP (IPC)" "CPU Frequency" "Branch Miss %"
printf "+-%-24s-+-%-18s-+-%-18s-+-%-16s-+\n" "------------------------" "------------------" "------------------" "----------------"
for idx in "${!TIERS[@]}"; do
    tier="${TIERS[$idx]}"
    label="${TIER_LABELS[$idx]}"
    printf "| %-24s | %-18s | %-18s | %-16s |\n" \
        "$label" \
        "${MAP_IPC[$tier]:-N/A}" \
        "${MAP_GHZ[$tier]:-N/A}" \
        "${MAP_BRANCH_MISS[$tier]:-N/A}"
done
printf "+-%-24s-+-%-18s-+-%-18s-+-%-16s-+\n" "------------------------" "------------------" "------------------" "----------------"
echo "================================================================================"
echo ""

# Write comprehensive grouped summary
{
    echo "ASHWA AVX-512BW CONTEXTUAL BENCHMARK REPORT"
    echo "Date:   $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Commit: $(git rev-parse HEAD)"
    echo "================================================================================"
    echo ""
    echo "[CONTEXT 1: SYSTEM, CACHE & MEMORY TOPOLOGY]"
    printf "| %-28s | %-45s |\n" "CPU Model" "$CPU_MODEL"
    printf "| %-28s | %-45s |\n" "Highest Available ISA" "$HIGHEST_ISA"
    printf "| %-28s | %-45s |\n" "L1 Data Cache (L1d)" "$L1D_CACHE"
    printf "| %-28s | %-45s |\n" "L1 Instruction Cache (L1i)" "$L1I_CACHE"
    printf "| %-28s | %-45s |\n" "L2 Cache" "$L2_CACHE"
    printf "| %-28s | %-45s |\n" "L3 Cache" "$L3_CACHE"
    printf "| %-28s | %-45s |\n" "STREAM Triad Best Rate" "$STREAM_TRIAD_RATE"
    printf "| %-28s | %-45s |\n" "STREAM Triad Best Time" "$STREAM_TRIAD_TIME"
    echo ""
    echo "[CONTEXT 2: AVX-512BW THROUGHPUT & LATENCY]"
    cat "${RESULTS_DIR}/throughput_latency.log" 2>/dev/null || true
    echo ""
    echo "[CONTEXT 3: INSTRUCTION-LEVEL PARALLELISM & HARDWARE METRICS]"
    printf "| %-24s | %-18s | %-18s | %-16s |\n" "Tier / Level" "ILP (IPC)" "CPU Frequency" "Branch Miss %"
    printf "|--------------------------|--------------------|--------------------|------------------|\n"
    for idx in "${!TIERS[@]}"; do
        tier="${TIERS[$idx]}"
        label="${TIER_LABELS[$idx]}"
        printf "| %-24s | %-18s | %-18s | %-16s |\n" \
            "$label" \
            "${MAP_IPC[$tier]:-N/A}" \
            "${MAP_GHZ[$tier]:-N/A}" \
            "${MAP_BRANCH_MISS[$tier]:-N/A}"
    done
} > "$SUMMARY_FILE"

echo "Complete benchmark logs saved to: $RESULTS_DIR"
