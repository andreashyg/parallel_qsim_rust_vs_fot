#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to run the travel time & summed departures extraction for a single experiment case.S
extract_travel_time_sum_dep_case() {
  local beta="$1"
  local java_seed_index="$2"
  local rust_seed="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_file_stem_pattern="$7"
  local input_file_format="$8"
  local tt_csv_path_pattern="${9}"
  local sd_csv_path_pattern="${10}"
  local num_parts="${11}"

  echo "Extracting average travel times and summed departures for parameters: replanning_variant=deprecated, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed"
  echo "writing into $tt_csv_path_pattern and $sd_csv_path_pattern"

  local rust_seed_string=()
  if [ "$rust_seed" == None ]; then
    rust_seed_string=("--no-random-seed")
  else
    rust_seed_string=("--use-random-seed" "$rust_seed")
  fi

  # Extract travel times and summed departures from the events written by the rust simulation.
  if ! ./target/release/event_data_extractor \
    --base-output-dir "$base_output_dir" \
    --experiment-set-name "$experiment_set_name" \
    --read-from-random "$java_seed_index" \
    "${rust_seed_string[@]}" \
    --input-file-stem-pattern "$input_file_stem_pattern" \
    --input-file-format "$input_file_format" \
    --tt-csv-path-pattern "$tt_csv_path_pattern" \
    --sd-csv-path-pattern "$sd_csv_path_pattern" \
    --num-parts "$num_parts" \
    --global-config-path "$global_config_path" \
    --link-to-path-map-name "braess" \
    --beta "$beta"
  then
    echo "Travel-time/summed departures extraction failed, continuing with next case." >&2
    record_failure extraction "deprecated" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f extract_travel_time_sum_dep_case