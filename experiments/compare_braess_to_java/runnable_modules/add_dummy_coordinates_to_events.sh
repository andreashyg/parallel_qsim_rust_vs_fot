#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to run the travel time & summed departures extraction for a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# Will read the events written by the simulation in the given output directory and write the average travel times per
# route to a CSV file in the analysis subdirectory of the output root directory.
add_dummy_coordinates_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_file_pattern="$7"
  local output_file_pattern="$8"

  echo "Adding dummy coordinates to events for parameters: replanning_variant=$replanning_variant, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed"
  echo "writing from $input_file_pattern and writing to $output_file_pattern"

  # folder name of the replanning variant in the original java data
#  local replanning_str
#  replanning_str=$(get_original_java_replanning_folder_string "$replanning_variant")

  # events file from the original java run
#  local original_input_file="${SIM_OUTPUT_BASE_DIR}/../../../../braess/refinement/no_spillback_scenario/${replanning_str}/beta${beta}/random${read_from_random}/beta${beta}random${read_from_random}.output_events.xml.gz"

  # output file stem for the events file with added dummy coordinates, which will be created by the
  # simple_dummy_coordinate_adder binary in the next step
  # This is needed because the original java events file does not contain coordinates, which are needed for the
  # travel time and summed departures extraction (Rust doesn't accept event files without coordinates).
  # local new_output_file_stem="${SIM_OUTPUT_BASE_DIR}/../../../../Abschlussarbeiten/2026/andreas-hygrell-rust-vs-fot/compare_braess_to_java/${replanning_variant}/java_event_files_with_added_dummy_coords/beta${beta}/random${read_from_random}/beta${beta}random${read_from_random}.output_events_with_dummy_coords"

  local rust_seed_string=()
  if [ $rust_seed == None ]; then
    rust_seed_string=("--no-random-seed")
  else
    rust_seed_string=("--use-random-seed" "$rust_seed")
  fi

  # Often, this will fail, since output events of the Java runs are often not available.
  if ! cargo run --release --bin simple_dummy_coordinate_adder -- \
    --input-file-pattern "${input_file_pattern}" \
    --output-file-pattern "${output_file_pattern}" \
    --base-output-dir "${base_output_dir}" \
    --replanning-variant "${replanning_variant}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}" \
    "${rust_seed_string[@]}"
  then
    echo "Failed to add dummy coordinates to the original Java output events." >&2
    record_failure dummy_coordinate_adding "$replanning_variant" "$beta" "$java_seed_index" "reading_original_java"
    return 1
  fi
}

export -f add_dummy_coordinates_case