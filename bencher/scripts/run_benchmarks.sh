#!/usr/bin/env bash

# ==============================================================================
# Ashwa Hardware & Benchmark Execution Runner
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CPU_CORE="${CPU_CORE:-2}"
RESULTS_DIR="${HOME}/results"

mkdir -p "$RESULTS_DIR"
SUMMARY_FILE="${RESULTS_DIR}/summary.txt"

if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

cd "$HOME/ashwa"

# NOTE: We isolate benchmark core, eliminate ASLR/NMI tick jitter, and lock CPU scaling governor
sudo -n sysctl -w kernel.randomize_va_space=0 >/dev/null 2>&1 || true
sudo -n sysctl -w kernel.nmi_watchdog=0 >/dev/null 2>&1 || true
sudo -n sysctl -w kernel.perf_event_paranoid=-1 >/dev/null 2>&1 || true
sudo -n sysctl -w kernel.kptr_restrict=0 >/dev/null 2>&1 || true

echo performance | sudo -n tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor >/dev/null 2>&1 || true
echo performance | sudo -n tee /sys/devices/system/cpu/cpufreq/policy*/energy_performance_preference >/dev/null 2>&1 || true

HOST_NAME=$(cat /etc/hostname 2>/dev/null || uname -n 2>/dev/null || echo "localhost")
GIT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")

# ==============================================================================
# HARDWARE TOPOLOGY & ISA RESOLUTION
# ==============================================================================

CPU_MODEL=$(lscpu | awk -F': +' '/Model name/ {print $2; exit}' || echo "Unknown CPU")
L1D_CACHE=$(lscpu | awk -F': +' '/L1d cache/ {print $2; exit}' || echo "N/A")
L1I_CACHE=$(lscpu | awk -F': +' '/L1i cache/ {print $2; exit}' || echo "N/A")
L2_CACHE=$(lscpu  | awk -F': +' '/L2 cache/ {print $2; exit}' || echo "N/A")
L3_CACHE=$(lscpu  | awk -F': +' '/L3 cache/ {print $2; exit}' || echo "N/A")

# NOTE: AVX-512BW vectorized SIMD and target-feature codegen requires rustc nightly toolchain.
CARGO_CMD="cargo"
HAS_AVX512BW=false

if grep -q "\bavx512bw\b" /proc/cpuinfo 2>/dev/null && grep -q "\bavx512f\b" /proc/cpuinfo 2>/dev/null; then
    HAS_AVX512BW=true
    HIGHEST_ISA="AVX-512BW (512-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native -C target-feature=+avx512bw,+avx512f"

    if command -v rustup >/dev/null 2>&1 && rustup toolchain list 2>/dev/null | grep -q nightly; then
        CARGO_CMD="cargo +nightly"
    elif cargo +nightly --version >/dev/null 2>&1; then
        CARGO_CMD="cargo +nightly"
    else
        # WARN: Hardware supports AVX-512BW but nightly toolchain is missing; attempting rustup installation
        rustup toolchain install nightly --profile minimal >/dev/null 2>&1 && CARGO_CMD="cargo +nightly" || true
    fi
elif grep -q "\bavx2\b" /proc/cpuinfo 2>/dev/null; then
    HIGHEST_ISA="AVX2 (256-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native"
elif grep -q "\bsse4_2\b" /proc/cpuinfo 2>/dev/null; then
    HIGHEST_ISA="SSE4.2 (128-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native"
elif grep -q "\bssse3\b" /proc/cpuinfo 2>/dev/null; then
    HIGHEST_ISA="SSSE3 (128-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native"
elif grep -q "\bsse2\b" /proc/cpuinfo 2>/dev/null; then
    HIGHEST_ISA="SSE2 (128-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native"
elif grep -q "\bneon\b" /proc/cpuinfo 2>/dev/null; then
    HIGHEST_ISA="ARM NEON (128-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native"
else
    HIGHEST_ISA="SWAR (64-bit Scalar Fallback)"
    TARGET_FLAG="-C target-cpu=native"
fi

RUSTFLAGS="${RUSTFLAGS:-$TARGET_FLAG}"
export RUSTFLAGS

# ==============================================================================
# STREAM MEMORY BANDWIDTH BASELINE
# ==============================================================================

STREAM_BIN="/tmp/ashwa_stream"
STREAM_TRIAD_RATE="N/A"
STREAM_TRIAD_TIME="N/A"

STREAM_SRC=""
for p in "${SCRIPT_DIR}/stream.c" "${HOME}/stream.c" "${HOME}/scripts/stream.c" "${HOME}/ashwa/bencher/scripts/stream.c"; do
    if [ -f "$p" ]; then
        STREAM_SRC="$p"
        break
    fi
done

if [ -n "$STREAM_SRC" ]; then
    gcc -O3 -march=native "$STREAM_SRC" -o "$STREAM_BIN" 2>/dev/null || true

    if [ -f "$STREAM_BIN" ]; then
        STREAM_OUT=$(taskset -c "$CPU_CORE" "$STREAM_BIN" 2>/dev/null || true)

        raw_rate=$(echo "$STREAM_OUT" | awk '/TRIAD_BEST_RATE_MB_S:/ {print $2}' || true)
        raw_time=$(echo "$STREAM_OUT" | awk '/TRIAD_MIN_TIME_S:/ {print $2}' || true)

        if [ -n "$raw_rate" ]; then
            rate_gb=$(awk -v r="$raw_rate" 'BEGIN { r_num = r + 0; if (r_num > 0) printf "%.2f GB/s", r_num/1024; else print "" }')
            [ -n "$rate_gb" ] && STREAM_TRIAD_RATE="$raw_rate MB/s ($rate_gb)"
        fi

        if [ -n "$raw_time" ]; then
            STREAM_TRIAD_TIME=$(awk -v t="$raw_time" 'BEGIN { t_num = t + 0; if (t_num > 0) printf "%.2f ms", t_num * 1000; else print "N/A" }')
        fi
    fi
fi

# ==============================================================================
# CONTEXT 1: TOPOLOGY REPORT GENERATION
# ==============================================================================

render_context_1() {
    cat <<EOF
================================================================================
               CONTEXT 1: SYSTEM, CACHE & MEMORY TOPOLOGY                       
================================================================================
+----------------------------------+--------------------------------------------+
| Component / Metric               | Specification / Value                      |
+----------------------------------+--------------------------------------------+
| CPU Model                        | $(printf '%-42s' "$CPU_MODEL") |
| Highest Available ISA            | $(printf '%-42s' "$HIGHEST_ISA") |
| L1 Data Cache (L1d)              | $(printf '%-42s' "$L1D_CACHE") |
| L1 Instruction Cache (L1i)       | $(printf '%-42s' "$L1I_CACHE") |
| L2 Cache                         | $(printf '%-42s' "$L2_CACHE") |
| L3 Cache                         | $(printf '%-42s' "$L3_CACHE") |
| STREAM Triad Best Rate           | $(printf '%-42s' "$STREAM_TRIAD_RATE") |
| STREAM Triad Best Time           | $(printf '%-42s' "$STREAM_TRIAD_TIME") |
+----------------------------------+--------------------------------------------+
EOF
}

echo "================================================================================"
echo "                   ASHWA BENCHMARK & HARDWARE PROFILING SUITE                   "
echo "================================================================================"
echo "Date:         $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "Host:         $HOST_NAME"
echo "Commit:       $GIT_COMMIT"
echo "Toolchain:    $CARGO_CMD"
echo "ISA Target:   $HIGHEST_ISA ($RUSTFLAGS)"
echo "Pinned Core:  CPU $CPU_CORE (via taskset -c $CPU_CORE)"
echo "================================================================================"
echo ""

render_context_1
echo ""

# ==============================================================================
# CONTEXT 2: THROUGHPUT & LATENCY BENCHMARK
# ==============================================================================

echo "================================================================================"
echo " [2/3] RUNNING THROUGHPUT & LATENCY BENCHMARK"
echo " Toolchain:      $CARGO_CMD"
echo " Target Feature: $HIGHEST_ISA ($RUSTFLAGS)"
echo " Payload Tiers:  L1 (32 KiB), L2 (512 KiB), L3 (16 MiB), RAM (256 MiB)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "================================================================================"

drop_caches() {
    sync && echo 3 | sudo -n tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true
}

drop_caches
$CARGO_CMD bench --no-run -p ashwa --bench one_throughput >/dev/null 2>&1 || true
taskset -c "$CPU_CORE" $CARGO_CMD bench -p ashwa --bench one_throughput -- --nocapture 2>&1 | tee "${RESULTS_DIR}/throughput_latency.log"
echo ""

# ==============================================================================
# CONTEXT 3: INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & HARDWARE METRICS
# ==============================================================================

echo "================================================================================"
echo " [3/3] MEASURING INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & CPU METRICS"
echo " Harness:        core/examples/one_ilp.rs"
echo " Toolchain:      $CARGO_CMD"
echo " Target Feature: $HIGHEST_ISA ($RUSTFLAGS)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "================================================================================"

$CARGO_CMD build --release -p ashwa --example one_ilp >/dev/null 2>&1
ILP_BIN="./target/release/examples/one_ilp"
[ ! -f "$ILP_BIN" ] && ILP_BIN="./target/release/one_ilp"

TIERS=("l1" "l2" "l3" "ram")
TIER_LABELS=("L1 Cache (32 KiB)" "L2 Cache (512 KiB)" "L3 Cache (16 MiB)" "RAM (256 MiB)")

declare -A MAP_IPC
declare -A MAP_GHZ
declare -A MAP_BRANCH_MISS

HAS_PERF=false
command -v perf >/dev/null 2>&1 && HAS_PERF=true

for idx in "${!TIERS[@]}"; do
    tier="${TIERS[$idx]}"
    drop_caches

    # NOTE: Single-pass profiling capturing TSC cycle estimates (stdout) and hardware PMU counters (stderr)
    PERF_LOG="/tmp/ashwa_perf_${tier}.log"
    if [ "$HAS_PERF" = "true" ]; then
        ILP_RUN=$(taskset -c "$CPU_CORE" perf stat -x ';' -e instructions,cycles,task-clock,branches,branch-misses -- "$ILP_BIN" "$tier" 2>"$PERF_LOG" || true)
    else
        ILP_RUN=$(taskset -c "$CPU_CORE" "$ILP_BIN" "$tier" 2>"$PERF_LOG" || true)
    fi

    tier_ipc=$(echo "$ILP_RUN" | awk -F'|' '/PROFILING_METRICS/ {for(i=1;i<=NF;i++) if($i ~ /^ipc:/) {split($i,a,":"); print a[2]}}' | tr -d ' ' || echo "")
    tier_ghz=$(echo "$ILP_RUN" | awk -F'|' '/PROFILING_METRICS/ {for(i=1;i<=NF;i++) if($i ~ /^ghz:/) {split($i,a,":"); print a[2]}}' | tr -d ' ' || echo "")

    ipc="N/A"
    [ -n "$tier_ipc" ] && [ "$tier_ipc" != "0.00" ] && ipc="${tier_ipc} insn/cyc"

    ghz="N/A"
    [ -n "$tier_ghz" ] && [ "$tier_ghz" != "0.00" ] && ghz="${tier_ghz} GHz"

    b_miss_pct="N/A"
    if [ -f "$PERF_LOG" ]; then
        perf_insn=$(awk -F';' '/instructions/ {print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        perf_cycles=$(awk -F';' '/\<cycles\>/ {print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        b_miss=$(awk -F';' '/branch-misses/ {print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        b_total=$(awk -F';' '/\<branches\>/ {print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")

        # NOTE: Prioritize kernel PMU hardware counters if available on bare-metal / supported hypervisor
        if [ -n "$perf_insn" ] && [ -n "$perf_cycles" ]; then
            pmu_ipc=$(awk -v i="$perf_insn" -v c="$perf_cycles" 'BEGIN { c_num = c + 0; i_num = i + 0; if (c_num > 0) printf "%.2f insn/cyc", i_num / c_num; else print "" }')
            [ -n "$pmu_ipc" ] && ipc="$pmu_ipc"
        fi

        if [ -n "$b_miss" ] && [ -n "$b_total" ]; then
            b_miss_pct=$(awk -v m="$b_miss" -v tot="$b_total" 'BEGIN { m_num = m + 0; tot_num = tot + 0; if (tot_num > 0) printf "%.3f%%", (m_num / tot_num) * 100; else print "0.000%" }')
        fi
        rm -f "$PERF_LOG"
    fi

    MAP_IPC["$tier"]="$ipc"
    MAP_GHZ["$tier"]="$ghz"
    MAP_BRANCH_MISS["$tier"]="$b_miss_pct"
done

render_context_3() {
    cat <<EOF
================================================================================
          CONTEXT 3: INSTRUCTION-LEVEL PARALLELISM & HARDWARE METRICS           
================================================================================
+--------------------------+--------------------+--------------------+------------------+
| Tier / Level             | ILP (IPC)          | CPU Frequency      | Branch Miss %    |
+--------------------------+--------------------+--------------------+------------------+
EOF
    for idx in "${!TIERS[@]}"; do
        tier="${TIERS[$idx]}"
        label="${TIER_LABELS[$idx]}"
        printf "| %-24s | %-18s | %-18s | %-16s |\n" \
            "$label" \
            "${MAP_IPC[$tier]:-N/A}" \
            "${MAP_GHZ[$tier]:-N/A}" \
            "${MAP_BRANCH_MISS[$tier]:-N/A}"
    done
    echo "+--------------------------+--------------------+--------------------+------------------+"
}

render_context_3
echo ""

# ==============================================================================
# CONSOLIDATED SUMMARY FILE EXPORT
# ==============================================================================

{
    echo "ASHWA CONTEXTUAL BENCHMARK REPORT"
    echo "Date:       $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Host:       $HOST_NAME"
    echo "Commit:     $GIT_COMMIT"
    echo "ISA Target: $HIGHEST_ISA"
    echo "Toolchain:  $CARGO_CMD"
    echo "================================================================================"
    echo ""
    render_context_1
    echo ""
    echo "[CONTEXT 2: THROUGHPUT & LATENCY]"
    cat "${RESULTS_DIR}/throughput_latency.log" 2>/dev/null || true
    echo ""
    render_context_3
} > "$SUMMARY_FILE"

echo "Complete benchmark logs saved to: $RESULTS_DIR"
