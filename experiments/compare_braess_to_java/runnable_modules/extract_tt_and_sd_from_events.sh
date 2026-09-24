#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to run the travel time & summed departures extraction for a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# Will read the events written by the simulation in the given output directory and write the average travel times per
# route to a CSV file in the analysis subdirectory of the output root directory.
extract_travel_time_sum_dep_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_file_stem_pattern="$7"
  local input_file_format="$8"
  local tt_csv_path_pattern="$9"
  local sd_csv_path_pattern="${10}"
  local num_parts="${11}"

  echo "Extracting average travel times and summed departures for parameters: replanning_variant=$replanning_variant, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed"
  echo "writing into $tt_csv_path_pattern and $sd_csv_path_pattern"

  # this has been moved into the event data extractor itself
#  local id_store_path_pattern_string
#  if [[ $input_file_format == "xml.gz" ]]; then
#    # for xml.gz input files, we don't need to specify an id store path pattern
#    id_store_path_pattern_string=()
#  elif [[ "${input_file_format}" == "binpb" ]]; then
#    # for binpb input files, we need to specify an id store path pattern; the id store is in the experiment output dir
#    id_store_path_pattern_string=(--id-store-path-pattern "${experiment_output_dir_pattern}"/output_ids.binpb)
#  else
#    echo "Invalid input file format given: $input_file_format" >&2
#    record_failure extraction "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
#    return 1
#  fi

  local rust_seed_string=()
  if [ "$rust_seed" == None ]; then
    rust_seed_string=("--no-random-seed")
  else
    rust_seed_string=("--use-random-seed" "$rust_seed")
  fi

  # Extract travel times and summed departures from the events written by the rust simulation.
  if ! cargo run --release --bin event_data_extractor -- \
    --base-output-dir "$base_output_dir" \
    --experiment-set-name "$experiment_set_name" \
    --replanning-variant "$replanning_variant" \
    --read-from-random "$java_seed_index" \
    "${rust_seed_string[@]}" \
    --input-file-stem-pattern "$input_file_stem_pattern" \
    --input-file-format "$input_file_format" \
    --tt-csv-path-pattern "$tt_csv_path_pattern" \
    --sd-csv-path-pattern "$sd_csv_path_pattern" \
    --num-parts "$num_parts" \
    --link-to-path-map-name "braess" \
    --beta "$beta"
#    "${id_store_path_pattern_string[@]}" \
#    --beta "$beta"
  then
    echo "Travel-time/summed departures extraction failed, continuing with next case." >&2
    record_failure extraction "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f extract_travel_time_sum_dep_case