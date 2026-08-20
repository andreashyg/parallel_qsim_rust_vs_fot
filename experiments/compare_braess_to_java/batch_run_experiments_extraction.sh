#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run the simulation and the extraction back-to-back for every case.
parse_common_args "$@"

# run every experiment in rust, extract measurements and plot per seed
run_for_each_replvar_variedseed_beta_seed_combo_parallel run_and_extract_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
