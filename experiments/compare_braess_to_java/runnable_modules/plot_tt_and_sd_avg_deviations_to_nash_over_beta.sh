#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_deviation_boxplots_over_beta_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_tt_csv_path_pattern="$7"
  local input_sd_csv_path_pattern="$8"
  local seeds_to_use_array_string="${9}"
  local betas_to_use_array_string="${10}"

  read -a seeds_to_use <<< "$seeds_to_use_array_string"

#  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/deviations/"


  echo "Plotting average travel times and summed departures  boxplots for parameters: replanning_variant=${replanning_variant}, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed with seeds to use = ${seeds_to_use[*]}"

  # Plot the average travel times and summed departures once per replanning variant.
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/tt_and_sd_dev_boxplots_to_nash_over_beta.py \
    "${replanning_variant}" "${experiment_set_name}" "${java_seed_index}" "${rust_seed}" "$base_output_dir" \
    "$input_tt_csv_path_pattern" "$input_sd_csv_path_pattern" "$betas_to_use_array_string" "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "all_betas" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

plot_deviation_scatterplots_over_beta_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_tt_csv_path_pattern="$7"
  local input_sd_csv_path_pattern="$8"
  local seeds_to_use_array_string="${9}"
  local betas_to_use_array_string="${10}"


  read -a seeds_to_use <<< "$seeds_to_use_array_string"

#  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/deviations/"


  echo "Plotting average travel times and summed departures  boxplots for parameters: replanning_variant=${replanning_variant}, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed with seeds to use = ${seeds_to_use[*]}"

  # Plot the average travel times and summed departures once per replanning variant.
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/tt_and_sd_dev_scatterplots_to_nash_over_beta.py \
    "${replanning_variant}" "${experiment_set_name}" "${java_seed_index}" "${rust_seed}" "$base_output_dir" \
    "$input_tt_csv_path_pattern" "$input_sd_csv_path_pattern" "$betas_to_use_array_string" "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "all_betas" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_deviation_boxplots_over_beta_case
export -f plot_deviation_scatterplots_over_beta_case