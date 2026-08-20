#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run the simulation and the extraction and plotting per seed back-to-back for every case.
# Then run plotting averages over all seeds for every case.
parse_common_args "$@"
# run every experiment in rust, extract measurements and plot per seed
run_for_each_replvar_variedseed_beta_seed_combo_parallel run_and_extract_and_plot_per_seed_casease
# try to extract measurements from the original java output events (will mostly fail because events are missing)
run_for_each_replvar_variedseed_beta_seed_combo_parallel try_extracting_measurements_from_java_case
# reformat the original java measurements into the same format as the rust measurements (because the above extraction mostly fails)
run_for_each_replvar_variedseed_beta_seed_combo_parallel reformat_original_java_extracted_measurements_case
# plot the original java data per seed
run_for_each_replvar_variedseed_beta_seed_combo_parallel plot_original_java_data_per_seed_case
# plot averages over all seeds (for each beta)
run_for_each_replvar_variedseed_beta_combo plot_avg_over_seeds_case
# plot averages over all seeds (for each beta) for the original java data
run_for_each_replvar_variedseed_beta_combo plot_original_java_data_avg_over_seeds_case
# plot overview plots over all betas (containing averages over seeds)
run_for_each_replvar_variedseed_combo plot_once_per_replanning_variant_case
# plot overview plots over all betas (containing averages over seeds) for the original java data
run_for_each_replvar_variedseed_combo plot_original_java_data_once_per_replanning_variant_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
