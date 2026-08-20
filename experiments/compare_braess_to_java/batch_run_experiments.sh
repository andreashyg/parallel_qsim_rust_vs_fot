#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

parse_common_args "$@"

# Run only the Rust simulations for the full experiment matrix.
run_for_each_replvar_variedseed_beta_seed_combo_parallel run_experiment_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
