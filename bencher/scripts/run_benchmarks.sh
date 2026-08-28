#!/usr/bin/env bash

# ==============================================================================
# Ashwa Hardware & Benchmark Execution Runner
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CPU_CORE="${CPU_CORE:-2}"
BENCH_TARGET="${BENCH_TARGET:-}"

normalize_target() {
    case "${1,,}" in
        1|one|one_throughput|search_one) echo "one" ;;
        2|two|two_throughput|search_two) echo "two" ;;
        *) echo "" ;;
    esac
}

if [ $# -gt 0 ] && [ -z "$BENCH_TARGET" ]; then
    BENCH_TARGET="$1"
fi

if [ -n "$BENCH_TARGET" ]; then
    BENCH_TARGET=$(normalize_target "$BENCH_TARGET")
fi

if [ -z "$BENCH_TARGET" ]; then
    if [ -t 0 ]; then
        echo "================================================================================"
        echo "                       ASHWA BENCHMARK SUITE SELECTION                          "
        echo "================================================================================"
        echo " Please select the benchmark suite to run (no default):"
        echo "   1) one - Single-byte search benchmark (search_one / one_throughput)"
        echo "   2) two - Two-byte search benchmark (search_two / two_throughput)"
        read -rp " Select benchmark suite [1/2 or one/two]: " user_choice
        BENCH_TARGET=$(normalize_target "$user_choice")
    fi
fi

if [ -z "$BENCH_TARGET" ]; then
    echo "Error: BENCH_TARGET is required (no default). Must be 'one' or 'two'."
    exit 1
fi

if [ "$BENCH_TARGET" = "one" ]; then
    SUITE_TITLE="search_one (Single-Byte Needle)"
    CORE_BENCH="one_throughput"
    NPM_NODE_BENCH="$HOME/ashwa/npm/benches/one_throughput.js"
    NPM_WASM_BENCH="$HOME/ashwa/npm/benches/wasm_throughput.js"
    PYPI_BENCH="$HOME/ashwa/pypi/benches/one_throughput.py"
    ILP_EXAMPLE="one_ilp"
else
    SUITE_TITLE="search_two (Two-Byte Needle)"
    CORE_BENCH="two_throughput"
    NPM_NODE_BENCH="$HOME/ashwa/npm/benches/two_throughput.js"
    NPM_WASM_BENCH="$HOME/ashwa/npm/benches/wasm_two_throughput.js"
    PYPI_BENCH="$HOME/ashwa/pypi/benches/two_throughput.py"
    ILP_EXAMPLE="two_ilp"
fi

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
TOTAL_THREADS=$(nproc 2>/dev/null || lscpu | awk -F': +' '/^CPU\(s\):/ {print $2; exit}' || grep -c ^processor /proc/cpuinfo 2>/dev/null || echo "N/A")
CORES_PER_SOCKET=$(lscpu | awk -F': +' '/Core\(s\) per socket:/ {print $2; exit}' || echo "")
SOCKETS=$(lscpu | awk -F': +' '/Socket\(s\):/ {print $2; exit}' || echo "1")
THREADS_PER_CORE=$(lscpu | awk -F': +' '/Thread\(s\) per core:/ {print $2; exit}' || echo "1")

if [ -n "$CORES_PER_SOCKET" ] && [ -n "$SOCKETS" ]; then
    TOTAL_CORES=$((CORES_PER_SOCKET * SOCKETS))
else
    TOTAL_CORES="$TOTAL_THREADS"
fi

L1D_CACHE=$(lscpu | awk -F': +' '/L1d cache/ {print $2; exit}' || echo "N/A")
L1I_CACHE=$(lscpu | awk -F': +' '/L1i cache/ {print $2; exit}' || echo "N/A")
L2_CACHE=$(lscpu  | awk -F': +' '/L2 cache/ {print $2; exit}' || echo "N/A")
L3_CACHE=$(lscpu  | awk -F': +' '/L3 cache/ {print $2; exit}' || echo "N/A")

ARCH_UNAME=$(uname -m 2>/dev/null || echo "unknown")
CARGO_CMD="cargo"
HAS_AVX512BW=false

if [ "$ARCH_UNAME" = "x86_64" ] && grep -q "\bavx512bw\b" /proc/cpuinfo 2>/dev/null && grep -q "\bavx512f\b" /proc/cpuinfo 2>/dev/null; then
    HAS_AVX512BW=true
    HIGHEST_ISA="AVX-512BW (512-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native -C target-feature=+avx512bw,+avx512f"

    if command -v rustup >/dev/null 2>&1 && rustup toolchain list 2>/dev/null | grep -q nightly; then
        CARGO_CMD="cargo +nightly"
    elif cargo +nightly --version >/dev/null 2>&1; then
        CARGO_CMD="cargo +nightly"
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
elif [ "$ARCH_UNAME" = "aarch64" ] || [ "$ARCH_UNAME" = "arm64" ] || grep -q -E "\b(asimd|neon)\b" /proc/cpuinfo 2>/dev/null; then
    HIGHEST_ISA="ARM NEON (128-bit SIMD)"
    TARGET_FLAG="-C target-cpu=native"
    CARGO_CMD="cargo"
else
    HIGHEST_ISA="SWAR (64-bit Scalar Fallback)"
    TARGET_FLAG="-C target-cpu=native"
    CARGO_CMD="cargo"
fi

RUSTFLAGS="${RUSTFLAGS:-$TARGET_FLAG}"
export RUSTFLAGS

NODE_VERSION="N/A"
if command -v node >/dev/null 2>&1; then
    NODE_VERSION=$(node --version 2>/dev/null || echo "N/A")
fi

PYTHON_VERSION="N/A"
if command -v python3 >/dev/null 2>&1; then
    PYTHON_VERSION=$(python3 --version 2>/dev/null | awk '{print $2}' || echo "N/A")
fi

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
--------------------------------------------------------------------------------
 [1/5] HARDWARE TOPOLOGY & MEMORY BANDWIDTH
--------------------------------------------------------------------------------
+----------------------------------+--------------------------------------------+
| Component / Metric               | Specification / Value                      |
+----------------------------------+--------------------------------------------+
| CPU Model                        | $(printf '%-42s' "$CPU_MODEL") |
| CPU Topology                     | $(printf '%-42s' "$TOTAL_CORES Cores / $TOTAL_THREADS Threads (vCPUs)") |
| Threads Per Core                 | $(printf '%-42s' "$THREADS_PER_CORE") |
| Highest Available ISA            | $(printf '%-42s' "$HIGHEST_ISA") |
| L1 Data Cache (L1d)              | $(printf '%-42s' "$L1D_CACHE") |
| L1 Instruction Cache (L1i)       | $(printf '%-42s' "$L1I_CACHE") |
| L2 Cache                         | $(printf '%-42s' "$L2_CACHE") |
| L3 Cache                         | $(printf '%-42s' "$L3_CACHE") |
| STREAM Triad Best Rate           | $(printf '%-42s' "$STREAM_TRIAD_RATE") |
| STREAM Triad Best Time           | $(printf '%-42s' "$STREAM_TRIAD_TIME") |
+----------------------------------+--------------------------------------------+
--------------------------------------------------------------------------------
EOF
}

echo "--------------------------------------------------------------------------------"
echo "                   ASHWA BENCHMARK & HARDWARE PROFILING SUITE                   "
echo "--------------------------------------------------------------------------------"
echo "Date:         $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "Host:         $HOST_NAME"
echo "Commit:       $GIT_COMMIT"
echo "Target Suite: $SUITE_TITLE"
echo "CPU Model:    $CPU_MODEL"
echo "CPU Cores:    $TOTAL_CORES physical cores, $TOTAL_THREADS threads ($THREADS_PER_CORE threads/core)"
echo "Toolchain:    $CARGO_CMD | Node.js: $NODE_VERSION | Python: $PYTHON_VERSION"
echo "ISA Target:   $HIGHEST_ISA ($RUSTFLAGS)"
echo "Pinned Core:  CPU $CPU_CORE (via taskset -c $CPU_CORE)"
echo "--------------------------------------------------------------------------------"
echo ""

render_context_1
echo ""

# ==============================================================================
# CONTEXT 2: RUST CORE: THROUGHPUT & LATENCY BENCHMARK
# ==============================================================================

echo "--------------------------------------------------------------------------------"
echo " [2/5] RUST CORE: THROUGHPUT & LATENCY BENCHMARK ($SUITE_TITLE)"
echo " Toolchain:      $CARGO_CMD"
echo " Target Feature: $HIGHEST_ISA ($RUSTFLAGS)"
echo " Payload Tiers:  L1 (32 KiB), L2 (512 KiB), L3 (16 MiB), RAM (256 MiB)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "--------------------------------------------------------------------------------"

drop_caches() {
    sync && echo 3 | sudo -n tee /proc/sys/vm/drop_caches >/dev/null 2>&1 || true
}

drop_caches
$CARGO_CMD bench -q --no-run -p ashwa --bench "$CORE_BENCH" >/dev/null 2>&1 || true
taskset -c "$CPU_CORE" $CARGO_CMD bench -q -p ashwa --bench "$CORE_BENCH" -- --nocapture 2>&1 | grep -v -E '^(Finished|Running|\s*$)' | tee "${RESULTS_DIR}/throughput_latency.log"
echo "--------------------------------------------------------------------------------"
echo ""

# ==============================================================================
# CONTEXT 3: NPM BINDINGS: THROUGHPUT & LATENCY BENCHMARK
# ==============================================================================

echo "--------------------------------------------------------------------------------"
echo " [3/5] NPM BINDINGS: THROUGHPUT & LATENCY BENCHMARK ($SUITE_TITLE)"
echo " Node.js:        $NODE_VERSION"
echo " Architecture:   $ARCH_UNAME"
echo " Payload Tiers:  L1 (32 KiB), L2 (512 KiB), L3 (16 MiB), RAM (256 MiB)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "--------------------------------------------------------------------------------"

# Ensure NPM native module is compiled for host architecture with matching RUSTFLAGS
if [ -d "$HOME/ashwa/npm" ]; then
    (
        cd "$HOME/ashwa/npm"
        (cd native && RUSTFLAGS="$RUSTFLAGS" $CARGO_CMD build --release >/dev/null 2>&1 || true)
        
        target_lib="../../target/release/libashwa_node.so"
        [ ! -f "$target_lib" ] && target_lib="../target/release/libashwa_node.so"
        [ ! -f "$target_lib" ] && target_lib="target/release/libashwa_node.so"

        if [ -f "$target_lib" ]; then
            if [ "$ARCH_UNAME" = "x86_64" ]; then
                cp -f "$target_lib" "native/index.linux-x64-gnu.node"
            elif [ "$ARCH_UNAME" = "aarch64" ] || [ "$ARCH_UNAME" = "arm64" ]; then
                cp -f "$target_lib" "native/index.linux-arm64-gnu.node"
            fi
        fi
    ) || true
fi

if command -v node >/dev/null 2>&1; then
    if [ -f "$NPM_NODE_BENCH" ]; then
        echo " >> Node.js / V8 Native N-API Throughput Benchmark:"
        drop_caches
        taskset -c "$CPU_CORE" node "$NPM_NODE_BENCH" 2>&1 | tee "${RESULTS_DIR}/npm_node_throughput_latency.log"
        echo ""
    fi

    if [ "$ARCH_UNAME" = "x86_64" ] && [ -f "$NPM_WASM_BENCH" ]; then
        if [ ! -f "$HOME/ashwa/npm/wasm/pkg/ashwa_wasm.js" ] && command -v wasm-pack >/dev/null 2>&1; then
            (cd "$HOME/ashwa/npm" && npm run build:wasm >/dev/null 2>&1 || true)
        fi

        if [ -f "$HOME/ashwa/npm/wasm/pkg/ashwa_wasm.js" ] && [ -f "$HOME/ashwa/npm/wasm/pkg/ashwa_wasm_bg.wasm" ]; then
            echo " >> WebAssembly SIMD128 Throughput Benchmark (x86_64 only):"
            drop_caches
            taskset -c "$CPU_CORE" node "$NPM_WASM_BENCH" 2>&1 | tee "${RESULTS_DIR}/npm_wasm_throughput_latency.log"
            echo ""
        else
            echo " >> WebAssembly SIMD128 package not available. Skipping WASM benchmark."
            echo ""
        fi
    fi
else
    echo " Node.js not found. Skipping NPM benchmarks."
fi
echo "--------------------------------------------------------------------------------"
echo ""

# ==============================================================================
# CONTEXT 4: PYTHON BINDINGS: THROUGHPUT & LATENCY BENCHMARK
# ==============================================================================

echo "--------------------------------------------------------------------------------"
echo " [4/5] PYTHON BINDINGS: THROUGHPUT & LATENCY BENCHMARK ($SUITE_TITLE)"
echo " Python:         $PYTHON_VERSION"
echo " Architecture:   $ARCH_UNAME"
echo " Payload Tiers:  L1 (32 KiB), L2 (512 KiB), L3 (16 MiB), RAM (256 MiB)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "--------------------------------------------------------------------------------"

# Ensure Python native C-extension is compiled for host architecture with matching RUSTFLAGS
if [ -d "$HOME/ashwa/pypi" ]; then
    (
        cd "$HOME/ashwa/pypi"
        RUSTFLAGS="$RUSTFLAGS" $CARGO_CMD build --release >/dev/null 2>&1 || true

        target_lib="../target/release/libashwa.so"
        [ ! -f "$target_lib" ] && target_lib="../../target/release/libashwa.so"
        [ ! -f "$target_lib" ] && target_lib="target/release/libashwa.so"

        if [ -f "$target_lib" ]; then
            mkdir -p python/ashwa
            cp -f "$target_lib" "python/ashwa/ashwa.so"
        fi
    ) || true
fi

if command -v python3 >/dev/null 2>&1; then
    if [ -f "$PYPI_BENCH" ]; then
        echo " >> Python / PyO3 Native C-Extension Throughput Benchmark:"
        drop_caches
        taskset -c "$CPU_CORE" python3 "$PYPI_BENCH" 2>&1 | tee "${RESULTS_DIR}/python_throughput_latency.log"
        echo ""
    fi
else
    echo " Python 3 not found. Skipping Python benchmarks."
fi
echo "--------------------------------------------------------------------------------"
echo ""

# ==============================================================================
# CONTEXT 5: INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & HARDWARE METRICS
# ==============================================================================

echo "--------------------------------------------------------------------------------"
echo " [5/5] MEASURING INSTRUCTION-LEVEL PARALLELISM (ILP / IPC) & CPU METRICS ($SUITE_TITLE)"
echo " Harness:        core/examples/${ILP_EXAMPLE}.rs"
echo " Toolchain:      $CARGO_CMD"
echo " Target Feature: $HIGHEST_ISA ($RUSTFLAGS)"
echo " CPU Pinning:    Core $CPU_CORE"
echo "--------------------------------------------------------------------------------"

$CARGO_CMD build -q --release -p ashwa --example "$ILP_EXAMPLE" >/dev/null 2>&1
ILP_BIN="./target/release/examples/${ILP_EXAMPLE}"
[ ! -f "$ILP_BIN" ] && ILP_BIN="./target/release/${ILP_EXAMPLE}"

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
        perf_task_clock=$(awk -F';' '/task-clock/ {gsub(/,/, ".", $1); print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9.]+' || echo "")
        b_miss=$(awk -F';' '/branch-misses/ {print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")
        b_total=$(awk -F';' '/\<branches\>/ {print $1}' "$PERF_LOG" | tr -d ' ' | grep -o -E '^[0-9]+' || echo "")

        # NOTE: Prioritize kernel PMU hardware counters if available on bare-metal / supported hypervisor
        if [ -n "$perf_insn" ] && [ -n "$perf_cycles" ]; then
            pmu_ipc=$(awk -v i="$perf_insn" -v c="$perf_cycles" 'BEGIN { c_num = c + 0; i_num = i + 0; if (c_num > 0) printf "%.2f insn/cyc", i_num / c_num; else print "" }')
            [ -n "$pmu_ipc" ] && ipc="$pmu_ipc"
        fi

        if [ -n "$perf_cycles" ] && [ -n "$perf_task_clock" ]; then
            pmu_ghz=$(awk -v c="$perf_cycles" -v t="$perf_task_clock" 'BEGIN {
                c_num = c + 0; t_num = t + 0;
                if (t_num > 0 && c_num > 0) {
                    ghz_val = c_num / (t_num * 1000000.0);
                    if (ghz_val >= 0.1 && ghz_val <= 10.0) printf "%.2f GHz", ghz_val;
                }
            }')
            [ -n "$pmu_ghz" ] && ghz="$pmu_ghz"
        fi

        if [ -n "$b_miss" ] && [ -n "$b_total" ]; then
            b_miss_pct=$(awk -v m="$b_miss" -v tot="$b_total" 'BEGIN { m_num = m + 0; tot_num = tot + 0; if (tot_num > 0) printf "%.3f%%", (m_num / tot_num) * 100; else print "0.000%" }')
        fi
        rm -f "$PERF_LOG"
    fi

    # Fallback to sysfs/lscpu clock if GHz measurement is missing or 0.00
    if [ "$ghz" = "N/A" ] || [ "$ghz" = "0.00 GHz" ] || [ -z "$ghz" ]; then
        freq_khz=$(cat "/sys/devices/system/cpu/cpu${CPU_CORE}/cpufreq/cpuinfo_cur_freq" 2>/dev/null || cat "/sys/devices/system/cpu/cpu${CPU_CORE}/cpufreq/scaling_cur_freq" 2>/dev/null || cat "/sys/devices/system/cpu/cpu${CPU_CORE}/cpufreq/cpuinfo_max_freq" 2>/dev/null || true)
        if [ -n "$freq_khz" ]; then
            ghz=$(awk -v k="$freq_khz" 'BEGIN { k_num = k + 0; if (k_num > 0) printf "%.2f GHz", k_num / 1000000.0 }')
        fi
    fi

    if [ "$ghz" = "N/A" ] || [ "$ghz" = "0.00 GHz" ] || [ -z "$ghz" ]; then
        cpu_mhz=$(lscpu | awk -F': +' '/CPU max MHz/ {print $2; exit} /CPU MHz/ {print $2; exit}' || true)
        if [ -n "$cpu_mhz" ]; then
            ghz=$(awk -v m="$cpu_mhz" 'BEGIN { m_num = m + 0; if (m_num > 0) printf "%.2f GHz", m_num / 1000.0 }')
        fi
    fi
    [ -z "$ghz" ] && ghz="N/A"

    MAP_IPC["$tier"]="$ipc"
    MAP_GHZ["$tier"]="$ghz"
    MAP_BRANCH_MISS["$tier"]="$b_miss_pct"
done

render_context_5() {
    cat <<EOF
--------------------------------------------------------------------------------
 [5/5] INSTRUCTION-LEVEL PARALLELISM & HARDWARE METRICS
--------------------------------------------------------------------------------
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
    echo "--------------------------------------------------------------------------------"
}

render_context_5
echo ""

# ==============================================================================
# CONSOLIDATED SUMMARY FILE EXPORT
# ==============================================================================

{
    echo "--------------------------------------------------------------------------------"
    echo "                        ASHWA CONTEXTUAL BENCHMARK REPORT                       "
    echo "--------------------------------------------------------------------------------"
    echo "Date:         $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Host:         $HOST_NAME"
    echo "Commit:       $GIT_COMMIT"
    echo "Target Suite: $SUITE_TITLE"
    echo "CPU Model:    $CPU_MODEL"
    echo "CPU Cores:    $TOTAL_CORES physical cores, $TOTAL_THREADS threads ($THREADS_PER_CORE threads/core)"
    echo "ISA Target:   $HIGHEST_ISA"
    echo "Toolchain:    $CARGO_CMD | Node.js: $NODE_VERSION | Python: $PYTHON_VERSION"
    echo "--------------------------------------------------------------------------------"
    echo ""
    render_context_1
    echo ""
    echo "--------------------------------------------------------------------------------"
    echo " [2/5] RUST CORE: THROUGHPUT & LATENCY BENCHMARK ($SUITE_TITLE)"
    echo "--------------------------------------------------------------------------------"
    cat "${RESULTS_DIR}/throughput_latency.log" 2>/dev/null || true
    echo "--------------------------------------------------------------------------------"
    echo ""
    echo "--------------------------------------------------------------------------------"
    echo " [3/5] NPM BINDINGS: THROUGHPUT & LATENCY BENCHMARK ($SUITE_TITLE)"
    echo "--------------------------------------------------------------------------------"
    if [ -f "${RESULTS_DIR}/npm_node_throughput_latency.log" ]; then
        echo " >> Node.js / V8 Native N-API:"
        cat "${RESULTS_DIR}/npm_node_throughput_latency.log" 2>/dev/null || true
        echo ""
    fi
    if [ -f "${RESULTS_DIR}/npm_wasm_throughput_latency.log" ]; then
        echo " >> WebAssembly SIMD128 (x86_64):"
        cat "${RESULTS_DIR}/npm_wasm_throughput_latency.log" 2>/dev/null || true
        echo ""
    fi
    echo "--------------------------------------------------------------------------------"
    echo ""
    echo "--------------------------------------------------------------------------------"
    echo " [4/5] PYTHON BINDINGS: THROUGHPUT & LATENCY BENCHMARK ($SUITE_TITLE)"
    echo "--------------------------------------------------------------------------------"
    if [ -f "${RESULTS_DIR}/python_throughput_latency.log" ]; then
        echo " >> Python / PyO3 Native C-Extension:"
        cat "${RESULTS_DIR}/python_throughput_latency.log" 2>/dev/null || true
        echo ""
    fi
    echo "--------------------------------------------------------------------------------"
    echo ""
    render_context_5
} > "$SUMMARY_FILE"

echo "Complete benchmark logs saved to: $RESULTS_DIR"

