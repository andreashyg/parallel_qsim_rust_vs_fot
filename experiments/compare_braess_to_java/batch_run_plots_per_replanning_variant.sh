#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run plotting per seed and averaged over all seeds back-to-back for every parameter combination.
parse_common_args "$@"

# plot overview plots over all betas (containing averages over seeds)
run_for_each_replvar_variedseed_combo plot_once_per_replanning_variant_case

# print all failures that occurred during the batch run, if any.
print_failure_summary
