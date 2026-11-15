#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to run the average link travel time extraction for a single experiment case.
extract_avg_link_travel_time_case() {
  local beta="$1"
  local java_seed_index="$2"
  local rust_seed="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_file_stem_pattern="$7"
  local input_file_format="$8"
  local tts_over_le_time_csv_path_pattern="${9}"
  local avg_tt_output_path_pattern="${10}"
  local num_parts="${11}"
  local bin_size="${12}"

  echo "Extracting average link travel times for parameters: replanning_variant=deprecated, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed, bin_size=$bin_size"
  echo "writing into $tts_over_le_time_csv_path_pattern and $avg_tt_output_path_pattern"

  local rust_seed_string=()
  if [ "$rust_seed" != None ]; then
    rust_seed_string+=("--use-random-seed" "$rust_seed")
  fi

  local read_from_random_seed_string=()
  if [ "$java_seed_index" != None ]; then
    read_from_random_seed_string+=("--read-from-random" "$java_seed_index")
  fi

  # Extract average link travel times from the events written by the rust simulation.
  if ! ./target/release/average_travel_time_extractor \
    --base-output-dir "$base_output_dir" \
    --experiment-set-name "$experiment_set_name" \
    "${read_from_random_seed_string[@]}" \
    "${rust_seed_string[@]}" \
    --input-file-stem-pattern "$input_file_stem_pattern" \
    --input-file-format "$input_file_format" \
    --tts-over-le-time-csv-path-pattern "$tts_over_le_time_csv_path_pattern" \
    --avg-tt-output-path-pattern "$avg_tt_output_path_pattern" \
    --num-parts "$num_parts" \
    --global-config-path "$global_config_path" \
    --beta "$beta" \
    --bin-size "$bin_size"
  then
    echo "Average link travel time extraction failed, continuing with next case." >&2
    record_failure extraction "deprecated" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f extract_avg_link_travel_time_case