#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run plotting for original java data

parse_common_args "$@"

# plot the original java data per seed
run_for_each_replvar_variedseed_beta_seed_combo_parallel plot_original_java_data_per_seed_case
# plot averages over all seeds (for each beta) for the original java data
run_for_each_replvar_variedseed_beta_combo plot_original_java_data_avg_over_seeds_case
# plot overview plots over all betas (containing averages over seeds) for the original java data
run_for_each_replvar_variedseed_combo plot_original_java_data_once_per_replanning_variant_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
