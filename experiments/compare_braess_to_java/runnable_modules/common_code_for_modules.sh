#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../failure_handling.sh"

# function to read config file entries into a variable.
# Read the config file at experiments/compare_braess_to_java/experiment_sets/global_config.yaml
# The first argument is the name of the variable to read into, the second argument is the key in the config file.
read_config_file_entry() {
  local -n var_to_change="$1"
  local key="$2"

  mapfile -t var_to_change < <(yq -r ".global_parameters.${key} | if type == \"array\" then .[] else . end" "$SCRIPT_DIR/../experiment_sets/global_config.yaml")
}

run_callback_for_parameter_arrays_parallel() {
  local -n replanning_variants_ref="$1"
  local -n betas_ref="$2"
  local -n java_seed_indices_ref="$3"
  local -n rust_seeds_ref="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local max_parallel_jobs="$7"
  local resume="$8"
  local resume_failed="$9"
  local script_type="${10}"
  local actual_callback="${11}"

  local batch_common_file="${SCRIPT_DIR}/common_code_for_modules.sh"

  shift 11

  local resume_arg=()
  if [[ "$resume" == "true" ]]; then
    resume_arg+=("--resume")
  fi
  if [[ "$resume_failed" == "true" ]]; then
    if [[ "$resume" == "true" ]]; then
      echo "Error: --resume-failed and --resume are mutually exclusive." >&2
      return 1
    fi
    resume_arg+=("--resume-failed")
  fi

  local callback_q experiment_set_name_q base_output_dir_q common_q arg extra_args=""

  # create a temporary file to log failures from parallel jobs
  local failure_log
  failure_log="$(mktemp)"

  # quote all the arguments for use in the parallel command
  printf -v callback_q '%q' "$actual_callback"
  printf -v experiment_set_name_q '%q' "$experiment_set_name"
  printf -v base_output_dir_q '%q' "$base_output_dir"
  printf -v common_q '%q' "$batch_common_file"

  for arg in "$@"; do
    extra_args+=" $(printf '%q' "$arg")"
  done
  printf "Extra arguments: %s\n" "$extra_args"

  printf "Running callback '%s' for all parameter combinations in parallel with up to %d jobs...\n" "$actual_callback" "$max_parallel_jobs"
  export FAILURE_LOG_FILE="$failure_log"

  local module_file="${SCRIPT_DIR}/run_rust_based_on_java_output.sh"
  printf -v module_q '%q' "$module_file"

  local joblog_path_pattern
  read_config_file_entry joblog_path_pattern joblog_path_pattern

  joblog_path="${joblog_path_pattern//\{experiment_set_name\}/$experiment_set_name}"
  joblog_path="${joblog_path//\{base_output_dir\}/$base_output_dir}"
  joblog_path="${joblog_path//\{callback\}/$callback_q}"

  mkdir -p "$(dirname "$joblog_path")"  # ensure the directory for the joblog exists

  if [[ "$script_type" == "rust" ]]; then
    echo "Building Rust binaries before running parallel jobs..."
    cargo build --release --bins
    echo "Finished building Rust binaries."
  elif [[ "$script_type" == "java" ]]; then
    echo "Building Java binaries before running parallel jobs..."
    JAVA_HOME=/home/andreas/.jdks/ms-25.0.4.1 ./java_matsim/mvnw -f ./java_matsim/pom.xml -DskipTests package
    echo "Finished building Java binaries."
  fi
  echo "Starting parallel execution of callback '$actual_callback' for all parameter combinations..."

  parallel --memsuspend 10G \
    --will-cite --eta --bar -j "${max_parallel_jobs}" \
    "${resume_arg[@]}" \
    --joblog "${joblog_path}" \
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