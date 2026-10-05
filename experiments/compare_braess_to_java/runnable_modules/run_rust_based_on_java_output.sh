#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to run a single experiment case.
run_experiment_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local delete_output_dir_if_existing="$7"

  echo "Running rust simulation with parameters: replanning_variant=${replanning_variant}, beta=${beta}, java_seed_index=${java_seed_index}, rust_seed=${rust_seed}"

  local delete_output_dir_arg=()
  if [ "$delete_output_dir_if_existing" = true ]; then
    delete_output_dir_arg=(--delete-output-dir-if-existing)
  fi

  # Run the Rust simulation; failures are reported but do not stop the batch.
  if ! ./target/release/run_single_braess_iter_from_java_output \
    --beta "$beta" \
    --read-from-random "$java_seed_index" \
    --use-random-seed "$rust_seed" \
    --replanning-variant "$replanning_variant" \
    --experiment-set-name "$experiment_set_name" \
    --base-output-dir "$base_output_dir" \
    "${delete_output_dir_arg[@]}"
  then
    echo "Experiment failed, continuing with next case." >&2
    # Record the failure with the appropriate parameters. Will append to the failure log file if specified (in the
    # global environment variable FAILURE_LOG_FILE).
    record_failure experiment "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f run_experiment_case

