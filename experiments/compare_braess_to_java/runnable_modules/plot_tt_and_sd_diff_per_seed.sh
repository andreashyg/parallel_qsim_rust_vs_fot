#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_diff_per_seed_case() {
  local beta="$1"
  local java_seed_index="$2"
  local rust_seed="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local main_input_tt_csv_path_pattern="$7"
  local main_input_sd_csv_path_pattern="$8"
  local secondary_input_tt_csv_path_pattern="${9}"
  local secondary_input_sd_csv_path_pattern="${10}"
  local minus_what="${11}"
  local sd_ybotlim="${12}"
  local sd_ytoplim="${13}"
  local tt_ybotlim="${14}"
  local tt_ytoplim="${15}"

  echo "Plotting differences of average travel times and summed departures per seed for parameters: replanning_variant=deprecated, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed, experiment_set_name=$experiment_set_name"


  # Plot the average travel times and summed departures.
  if ! python python_plotting/braess/plot_sd_tt_single_seed_differences.py \
    "$beta" "$java_seed_index" "$rust_seed" "$experiment_set_name" "$base_output_dir" \
    "$minus_what" \
    "$main_input_tt_csv_path_pattern" "$main_input_sd_csv_path_pattern" \
    "$secondary_input_tt_csv_path_pattern" "$secondary_input_sd_csv_path_pattern" \
    "$global_config_path" \
    "$sd_ybotlim" "$sd_ytoplim" "$tt_ybotlim" "$tt_ytoplim"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "deprecated" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_diff_per_seed_case