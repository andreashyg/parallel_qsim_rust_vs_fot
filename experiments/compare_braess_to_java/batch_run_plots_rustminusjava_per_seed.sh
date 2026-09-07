#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run plotting per seed and averaged over all seeds back-to-back for every parameter combination.
parse_common_args "$@"

# plot per seed
run_for_each_replvar_variedseed_beta_seed_combo_parallel plot_differences_per_seed_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
