#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_avgd_over_seeds_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_tt_csv_path_pattern="$7"
  local input_sd_csv_path_pattern="$8"
  local seeds_to_use_array_string="${9}"


  read -a seeds_to_use <<< "$seeds_to_use_array_string"

  echo "Plotting average travel times and summed departures averaged over seeds for parameters: replanning_variant=${replanning_variant}, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed with seeds to use = ${seeds_to_use[*]}"

  # Plot the average travel times and summed departures averaged over seeds.
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/tt_and_sd_avg_over_seeds.py \
    "$beta" "${replanning_variant}" "$experiment_set_name" "$java_seed_index" "$rust_seed" \
    "$base_output_dir" "$input_tt_csv_path_pattern" "$input_sd_csv_path_pattern" "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_avgd_over_seeds_case