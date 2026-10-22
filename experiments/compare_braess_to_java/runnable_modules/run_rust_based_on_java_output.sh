#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to run a single experiment case.
run_experiment_case() {
  local beta="$1"
  local java_seed_index="$2"
  local rust_seed="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local delete_output_dir_if_existing="$7"

  echo "Running rust simulation with parameters: replanning_variant=deprecated, beta=${beta}, java_seed_index=${java_seed_index}, rust_seed=${rust_seed}"

  local delete_output_dir_arg=()
  if [ "$delete_output_dir_if_existing" = true ]; then
    delete_output_dir_arg=(--delete-output-dir-if-existing)
  fi

  # Run the Rust simulation; failures are reported but do not stop the batch.
  if ! ./target/release/run_single_braess_iter_from_java_output \
    --beta "$beta" \
    --read-from-random "$java_seed_index" \
    --use-random-seed "$rust_seed" \
    --experiment-set-name "$experiment_set_name" \
    --base-output-dir "$base_output_dir" \
    --global-config-path "$global_config_path" \
    "${delete_output_dir_arg[@]}"
  then
    echo "Experiment failed, continuing with next case." >&2
    # Record the failure with the appropriate parameters. Will append to the failure log file if specified (in the
    # global environment variable FAILURE_LOG_FILE).
    record_failure experiment "deprecated" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f run_experiment_case

