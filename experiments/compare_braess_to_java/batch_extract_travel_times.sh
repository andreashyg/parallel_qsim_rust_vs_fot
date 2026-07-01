#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Extract travel times only, assuming the simulation outputs already exist.
parse_common_args "$@"
for_each_experiment_case extract_travel_time_case
print_failure_summary
