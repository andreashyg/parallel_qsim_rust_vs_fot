#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../failure_handling.sh"

run_callback_for_parameter_arrays_parallel() {
  local -n replanning_variants_ref="$1"
  local -n betas_ref="$2"
  local -n java_seed_indices_ref="$3"
  local -n rust_seeds_ref="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
#  local experiment_output_dir_pattern="$7"
#  local delete_output_dir_if_existing="$8"
  local max_parallel_jobs="$7"
  local actual_callback="$8"

  # local batch_common_file="${SCRIPT_DIR}/../batch_common.sh"
  local batch_common_file="${SCRIPT_DIR}/common_code_for_modules.sh"

  shift 8

  local callback_q experiment_set_name_q base_output_dir_q common_q arg extra_args=""

  # create a temporary file to log failures from parallel jobs
  local failure_log
  failure_log="$(mktemp)"

  # quote all the arguments for use in the parallel command
  printf -v callback_q '%q' "$actual_callback"
  printf -v experiment_set_name_q '%q' "$experiment_set_name"
  printf -v base_output_dir_q '%q' "$base_output_dir"
#  printf -v experiment_output_dir_pattern_q '%q' "$experiment_output_dir_pattern"
#  printf -v delete_output_dir_if_existing_q '%q' "$delete_output_dir_if_existing"
  printf -v common_q '%q' "$batch_common_file"

  for arg in "$@"; do
    extra_args+=" $(printf '%q' "$arg")"
  done
  printf "Extra arguments: %s\n" "$extra_args"

  printf "Running callback '%s' for all parameter combinations in parallel with up to %d jobs...\n" "$actual_callback" "$max_parallel_jobs"
  export FAILURE_LOG_FILE="$failure_log"

  local module_file="${SCRIPT_DIR}/run_rust_based_on_java_output.sh"
  printf -v module_q '%q' "$module_file"

 # TODO at some point, add --memfree *smth* to parallel to avoid running out of memory when running many jobs in parallel.

  parallel --will-cite --eta --bar -j "${max_parallel_jobs}" \
    "bash -lc 'source ${common_q}; source ${module_q}; ${callback_q} {1} {2} {3} {4} ${experiment_set_name_q} ${base_output_dir_q} ${extra_args}'" \
    ::: "${replanning_variants_ref[@]}" \
    ::: "${betas_ref[@]}" \
    ::: "${java_seed_indices_ref[@]}" \
    ::: "${rust_seeds_ref[@]}"

  printf "Finished running callback '%s' for all parameter combinations in parallel.\n" "$actual_callback"

  # import any failures that occurred during the parallel execution from file into the global variables
  import_failure_log "$failure_log"
  rm -f "$failure_log"  # remove the temporary file after importing the failures
  unset FAILURE_LOG_FILE  # unset the environment variable to avoid accidental use later

  # Note: we don't print the failure summary here, this is done in the main script after all parallel runs are finished.
  return 0
}