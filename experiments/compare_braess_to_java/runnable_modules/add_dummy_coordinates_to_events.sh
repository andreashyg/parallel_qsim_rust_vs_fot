#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

add_dummy_coordinates_case() {
  local replanning_variant="$1"
  local beta="$2"
  local read_from_random_seed_index="$3"
  local use_random_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_file_pattern="$7"
  local output_file_pattern="$8"

  echo "Adding dummy coordinates to events for parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random_seed_index=$read_from_random_seed_index, use_random_seed=$use_random_seed"
  echo "writing from $input_file_pattern and writing to $output_file_pattern"

  local use_random_seed_string=()
  if [ $use_random_seed == None ]; then
    use_random_seed_string=("--no-use-random-seed")
  else
    use_random_seed_string=("--use-random-seed" "$use_random_seed")
  fi

  # Often, this will fail, since output events of the Java runs are often not available.
  if ! ./target/release/simple_dummy_coordinate_adder \
    --input-file-pattern "${input_file_pattern}" \
    --output-file-pattern "${output_file_pattern}" \
    --base-output-dir "${base_output_dir}" \
    --replanning-variant "${replanning_variant}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${read_from_random_seed_index}" \
    "${use_random_seed_string[@]}"
  then
    echo "Failed to add dummy coordinates to the original Java output events." >&2
    record_failure dummy_coordinate_adding "$replanning_variant" "$beta" "$read_from_random_seed_index" "$use_random_seed"
    return 1
  fi
}

export -f add_dummy_coordinates_case