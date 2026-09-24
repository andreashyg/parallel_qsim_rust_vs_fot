#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

reformat_original_java_extracted_measurements_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local _rust_seed="$4"  # unused in this module
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local original_tt_tsv_file_pattern="$7"
  local original_sd_tsv_file_pattern="$8"
  local new_tt_csv_path_pattern="$9"
  local new_sd_csv_path_pattern="${10}"

  echo "Copying and reformatting extracted original java average travel times and summed departures for parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random=${java_seed_index}"
  echo "writing into $new_tt_csv_path_pattern and $new_sd_csv_path_pattern"

  if ! cargo run --release --bin simple_csv_column_renamer -- \
    --input-file-pattern "${original_tt_tsv_file_pattern}" \
    --output-file-pattern "${new_tt_csv_path_pattern}" \
    --base-output-dir "${base_output_dir}" \
    --replanning-variant "${replanning_variant}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}" \
    --column-name departure_time \
    --column-name avg_travel_time_path_0 --column-name avg_travel_time_path_1 \
    --column-name avg_travel_time_path_2 --column-name avg_travel_time || ! cargo run --release --bin \
    simple_csv_column_renamer -- \
    --input-file-pattern "${original_sd_tsv_file_pattern}" \
    --output-file-pattern "${new_sd_csv_path_pattern}" \
    --base-output-dir "${base_output_dir}" \
    --replanning-variant "${replanning_variant}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}" \
    --column-name time --column-name sum_departures_path_0 \
    --column-name sum_departures_path_1 --column-name sum_departures_path_2 \
    --column-name sum_departures_total
  then
    echo "Failed to copy and reformat the original java extracted data." >&2
    record_failure reformatting "$replanning_variant" "$beta" "$java_seed_index" "reading_original_java"
    return 1
  fi

}

export -f reformat_original_java_extracted_measurements_case