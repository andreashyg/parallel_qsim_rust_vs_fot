#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Reformat the original extracted measurements from the Java experiments to match the format of
# extracted measurements from the Rust experiments.
parse_common_args "$@"

run_for_each_replvar_variedseed_beta_seed_combo reformat_original_java_extracted_measurements_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
