#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${SCRIPT_DIR}/batch_common.sh"

# Run the simulation and the extraction back-to-back for every case.
parse_common_args "$@"
for_each_experiment_case run_and_extract_case per_seed
print_failure_summary
