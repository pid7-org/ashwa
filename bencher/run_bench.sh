#!/bin/bash
set -euo pipefail

# This script delegates to the root benchmark runner
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/../bench-aws" "$@"
