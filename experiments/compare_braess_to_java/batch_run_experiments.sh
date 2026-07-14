#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run only the Rust simulations for the full experiment matrix.
parse_common_args "$@"
for_each_experiment_case run_experiment_case per_seed
print_failure_summary
