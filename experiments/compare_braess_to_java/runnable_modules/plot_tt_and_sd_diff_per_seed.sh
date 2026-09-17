#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_diff_per_seed_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local _experiment_output_dir_pattern="$7"  # unused in this module
  local _delete_output_dir_if_existing="$8"  # unused in this module
  local output_tt_plot_path_pattern="$9"
  local output_sd_plot_path_pattern="${10}"
  local main_input_tt_csv_path_pattern="${11}"
  local main_input_sd_csv_path_pattern="${12}"
  local secondary_input_tt_csv_path_pattern="${13}"
  local secondary_input_sd_csv_path_pattern="${14}"

#  # directory where the plots will be written to.
#  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/per_seed"
#
#  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
#    echo "Skipping plotting per seed because output plots directory already exists: $output_plots_dir"
#    return 0
#  fi

  echo "Plotting differences of average travel times and summed departures per seed for parameters: replanning_variant=${replanning_variant}, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed, experiment_set_name=$experiment_set_name"


  # Plot the average travel times and summed departures.
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/plot_sd_tt_single_seed_differences.py \
    "$beta" "${replanning_variant}" "$java_seed_index" "$rust_seed" "$experiment_set_name" "$base_output_dir" \
    "$output_tt_plot_path_pattern" "$output_sd_plot_path_pattern" \
    "$main_input_tt_csv_path_pattern" "$main_input_sd_csv_path_pattern" \
    "$secondary_input_tt_csv_path_pattern" "$secondary_input_sd_csv_path_pattern"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_diff_per_seed_case