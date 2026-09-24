#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_avgd_over_seeds_diff_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local main_input_tt_csv_path_pattern="$7"
  local main_input_sd_csv_path_pattern="$8"
  local secondary_input_tt_csv_path_pattern="$9"
  local secondary_input_sd_csv_path_pattern="${10}"
  local seeds_to_use_array_string="${11}"
  local minus_what="${12}"

  read -a seeds_to_use <<< "$seeds_to_use_array_string"

#  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/avg_over_${seeds_to_avg_over}_seeds"

  echo "Plotting differences of average travel times and summed departures averaged over seeds for parameters: replanning_variant=${replanning_variant}, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed, experiment_set_name=$experiment_set_name with seeds to use = ${seeds_to_use[*]}"


  # Plot the average travel times and summed departures averaged over seeds.
  # if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/plot_tt_and_sd_avg_over_rust_minus_java.py "$beta" "${replanning_variant}" "$fixed_seed" "recreating_java_results" "varying_rust_seeds" "$output_plots_dir"
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/plot_tt_and_sd_avg_over_seeds_differences.py \
    "$beta" "${replanning_variant}" "$experiment_set_name" "$java_seed_index" "$rust_seed" "$base_output_dir" \
    "$minus_what" \
    "$main_input_tt_csv_path_pattern" "$main_input_sd_csv_path_pattern" \
    "$secondary_input_tt_csv_path_pattern" "$secondary_input_sd_csv_path_pattern" "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2

    record_failure plotting "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_avgd_over_seeds_diff_case