#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run plotting per seed and averaged over all seeds back-to-back for every parameter combination.
parse_common_args "$@"
for_each_experiment_case plot_once_per_replanning_variant_case once_per_replanning_variant
print_failure_summary
