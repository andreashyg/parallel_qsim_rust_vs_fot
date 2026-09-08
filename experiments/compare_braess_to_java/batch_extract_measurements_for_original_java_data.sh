#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Extract travel times from original java output events.
parse_common_args "$@"

run_for_each_replvar_variedseed_beta_seed_combo_parallel try_extracting_measurements_from_java_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
