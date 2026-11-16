#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_avg_link_tt_deviation_to_diff_run_boxplot() {
  local beta="$1"
  local java_seed_index="$2"
  local rust_seed="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local main_input_tt_csv_path_pattern="$7"
  local secondary_input_tt_csv_path_pattern="${8}"
  local seeds_to_use_array_string="${9}"
  local betas_to_use_array_string="${10}"
  local which_deviation="${11}"
  local tt_ybotlim="${12}"
  local tt_ytoplim="${13}"

  read -a seeds_to_use <<< "$seeds_to_use_array_string"

  echo "Plotting average link travel times deviation $which_deviation boxplot for parameters: replanning_variant=deprecated, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed with seeds to use = ${seeds_to_use[*]}"

  # Plot the average travel times and summed departures once per replanning variant.
  if ! python -m python_plotting.dresden.plot_link_tt_deviation_to_specific_run_boxplot \
    "${experiment_set_name}" "${java_seed_index}" "${rust_seed}" "$base_output_dir" \
    "$which_deviation" \
    "$main_input_tt_csv_path_pattern" \
    "$secondary_input_tt_csv_path_pattern" \
    "$betas_to_use_array_string" "$global_config_path" \
    "$tt_ybotlim" "$tt_ytoplim" \
    "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "deprecated" "all_betas" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}


export -f plot_avg_link_tt_deviation_to_diff_run_boxplot
