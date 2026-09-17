#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

parse_common_args "$@"

# Run only the Rust simulations for the full experiment matrix.
run_for_each_replvar_variedseed_beta_seed_combo run_java_single_iteration_experiment_case
#run_java_single_iteration_experiment_case sel-exp10-switch-at80 "rust" 64 1
# (will take read_from_random=JAVA_SEED_INDEX_WHEN_FIXED=1 and use_random_seed=RUST_SEEDS_TO_ITERTATE_OVER[1] = 42)

# print all failures that occurred during the batch run, if any.
print_failure_summary
