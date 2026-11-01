#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

#get_original_java_replanning_folder_string() {
#  local replanning_variant="$1"
#
#  if [ "${replanning_variant}" = "sel-exp1-switch-at50" ]; then
#    echo "2026-05-8-12-16-8_500it_reRouteProba0.1until0.5it_selExpBeta1proba0.9_msaFrom0.5it"
#  elif [ "${replanning_variant}" = "sel-exp1-switch-at80" ]; then
#    echo "2026-05-10-10-2-21_500it_reRouteProba0.1until0.8it_selExpBeta1proba0.9_msaFrom0.8it"
#  elif [ "${replanning_variant}" = "sel-exp10-switch-at80" ]; then
#    echo "2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it"
#  elif [ "${replanning_variant}" = "sel-exp10-switch-at80-my-own" ]; then
#    echo "2026-10-14-12-22-10_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it_myFirstReplRun"
#  elif [ "${replanning_variant}" = "sel-exp10-switch-at80-my-own-only-leg-scoring-and-brain1-only64" ]; then
#    echo "2026-10-21-23-11-32_original_settings_but_only_leg_scoring_and_brain_1_only_beta_64"
#  else
#    echo "unknown replanning_variant value: $replanning_variant" >&2
#    return 1
#  fi
#}

prepare_original_population_case() {
  local beta="$1"
  local java_seed_index="$2"
  local _rust_seed="$3"  # unused in this module
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local skip_if_existing="$7"
  local pop_to_be_changed_pattern="$8"
  local pop_with_correct_activity_times_pattern="${9}"
  local vehicle_file_pattern="${10}"
  local add_access_egress_legs_if_necessary="${11}"
  local output_file_pattern="${12}"

  local skip_if_existing_array=()
  if [[ "$skip_if_existing" == "true" ]]; then
    skip_if_existing_array+=("--skip-if-existing")
  fi

  local pop_with_correct_activity_times_pattern_array=()
  if [[ "$pop_with_correct_activity_times_pattern" != "None" ]]; then
    pop_with_correct_activity_times_pattern_array+=(--pop-with-correct-activity-times-pattern "${pop_with_correct_activity_times_pattern}")
  fi

  local add_access_egress_legs_if_necessary_array=()
  if [[ "$add_access_egress_legs_if_necessary" == "true" ]]; then
    add_access_egress_legs_if_necessary_array+=(--add-access-egress-legs-if-necessary)
  fi

  echo "Preparing original population with parameters: replanning_variant=deprecated, beta=$beta, java_seed_index=$java_seed_index"

#  # folder name of the replanning variant in the original java data
#  replanning_variant_original=$(get_original_java_replanning_folder_string "$replanning_variant")

#  if ! cargo run --release --bin activity_time_replacer -- \
  if ! ./target/release/original_population_preparer \
    --base-output-dir "${base_output_dir}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}" \
    --global-config-path "${global_config_path}" \
    --pop-to-be-changed-pattern "${pop_to_be_changed_pattern}" \
    "${pop_with_correct_activity_times_pattern_array[@]}" \
    --vehicle-file-pattern "${vehicle_file_pattern}" \
    "${add_access_egress_legs_if_necessary_array[@]}" \
    --output-file-pattern "${output_file_pattern}" \
    "${skip_if_existing_array[@]}"
  then
    echo "Failed to replace activity times." >&2
    record_failure activity_time_replacement "deprecated" "$beta" "$java_seed_index" "None"
    return 1
  fi
}

export -f prepare_original_population_case
#export -f get_original_java_replanning_folder_string