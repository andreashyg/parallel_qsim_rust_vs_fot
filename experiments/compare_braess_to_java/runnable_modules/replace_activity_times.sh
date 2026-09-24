#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

get_original_java_replanning_folder_string() {
  local replanning_variant="$1"

  if [ "${replanning_variant}" = "sel-exp1-switch-at50" ]; then
    echo "2026-05-8-12-16-8_500it_reRouteProba0.1until0.5it_selExpBeta1proba0.9_msaFrom0.5it"
  elif [ "${replanning_variant}" = "sel-exp1-switch-at80" ]; then
    echo "2026-05-10-10-2-21_500it_reRouteProba0.1until0.8it_selExpBeta1proba0.9_msaFrom0.8it"
  elif [ "${replanning_variant}" = "sel-exp10-switch-at80" ]; then
    echo "2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it"
  else
    echo "unknown replanning_variant value: $replanning_variant" >&2
    return 1
  fi
}

replace_activity_times_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local _rust_seed="$4"  # unused in this module
  local experiment_set_name="$5"
  local base_output_dir="$6"

  echo "Replacing activity times with parameters: replanning_variant=$replanning_variant, beta=$beta, java_seed_index=$java_seed_index"

  # folder name of the replanning variant in the original java data
  replanning_variant_original=$(get_original_java_replanning_folder_string "$replanning_variant")

  if ! cargo run --release --bin activity_time_replacer -- \
    --base-output-dir "${base_output_dir}" \
    --replanning-variant-original "${replanning_variant_original}" \
    --replanning-variant "${replanning_variant}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}"
  then
    echo "Failed to replace activity times." >&2
    record_failure activity_time_replacement "$replanning_variant" "$beta" "$java_seed_index" "None"
    return 1
  fi
}

export -f replace_activity_times_case
export -f get_original_java_replanning_folder_string