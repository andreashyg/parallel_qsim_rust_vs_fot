#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_avgd_over_seeds_diff_case() {
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
  local seeds_to_use_array_string="${11}"
  local minus_what="${12}"
  local sd_ybotlim="${13}"
  local sd_ytoplim="${14}"
  local tt_ybotlim="${15}"
  local tt_ytoplim="${16}"

  read -a seeds_to_use <<< "$seeds_to_use_array_string"

  echo "Plotting differences of average travel times and summed departures averaged over seeds for parameters: replanning_variant=deprecated, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed, experiment_set_name=$experiment_set_name with seeds to use = ${seeds_to_use[*]}"


  # Plot the average travel times and summed departures averaged over seeds.
  if ! python python_plotting/braess/plot_tt_and_sd_avg_over_seeds_differences.py \
    "$beta" "$experiment_set_name" "$java_seed_index" "$rust_seed" "$base_output_dir" \
    "$minus_what" \
    "$main_input_tt_csv_path_pattern" "$main_input_sd_csv_path_pattern" \
    "$secondary_input_tt_csv_path_pattern" "$secondary_input_sd_csv_path_pattern" "$global_config_path" \
    "$sd_ybotlim" "$sd_ytoplim" "$tt_ybotlim" "$tt_ytoplim" \
    "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2

    record_failure plotting "deprecated" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_avgd_over_seeds_diff_case