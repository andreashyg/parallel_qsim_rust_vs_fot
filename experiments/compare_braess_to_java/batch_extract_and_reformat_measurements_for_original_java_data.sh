#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Extract travel times only, assuming the simulation outputs already exist.
parse_common_args "$@"

run_for_each_replvar_variedseed_beta_seed_combo try_extracting_measurements_from_java_case
run_for_each_replvar_variedseed_beta_seed_combo reformat_original_java_extracted_measurements_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
