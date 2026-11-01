#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

reformat_original_java_extracted_measurements_case() {
  local beta="$1"
  local java_seed_index="$2"
  local _rust_seed="$3"  # unused in this module
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local original_tt_tsv_file_pattern="$7"
  local original_sd_tsv_file_pattern="$8"
  local new_tt_csv_path_pattern="${9}"
  local new_sd_csv_path_pattern="${10}"

  echo "Copying and reformatting extracted original java average travel times and summed departures for parameters: replanning_variant=deprecated, beta=$beta, read_from_random=${java_seed_index}"
  echo "writing into $new_tt_csv_path_pattern and $new_sd_csv_path_pattern"

  if ! ./target/release/simple_csv_column_renamer \
    --input-file-pattern "${original_tt_tsv_file_pattern}" \
    --output-file-pattern "${new_tt_csv_path_pattern}" \
    --base-output-dir "${base_output_dir}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}" \
    --global-config-path "${global_config_path}" \
    --column-name departure_time \
    --column-name avg_travel_time_path_0 --column-name avg_travel_time_path_1 \
    --column-name avg_travel_time_path_2 --column-name avg_travel_time || ! ./target/release/simple_csv_column_renamer \
    --input-file-pattern "${original_sd_tsv_file_pattern}" \
    --output-file-pattern "${new_sd_csv_path_pattern}" \
    --base-output-dir "${base_output_dir}" \
    --experiment-set-name "${experiment_set_name}" \
    --beta "${beta}" \
    --read-from-random "${java_seed_index}" \
    --global-config-path "${global_config_path}" \
    --column-name time --column-name sum_departures_path_0 \
    --column-name sum_departures_path_1 --column-name sum_departures_path_2 \
    --column-name sum_departures_total
  then
    echo "Failed to copy and reformat the original java extracted data." >&2
    record_failure reformatting "deprecated" "$beta" "$java_seed_index" "reading_original_java"
    return 1
  fi

}

export -f reformat_original_java_extracted_measurements_case