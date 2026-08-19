#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run plotting per seed and averaged over all seeds back-to-back for every parameter combination.
parse_common_args "$@"

# plot averages over all seeds (for each beta)
run_for_each_replvar_variedseed_beta_combo plot_avg_over_seeds_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
