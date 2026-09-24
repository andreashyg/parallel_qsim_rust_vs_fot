#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_per_seed_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local rust_seed="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_tt_csv_path_pattern="$7"
  local input_sd_csv_path_pattern="$8"

  # directory where the plots will be written to.
#  local output_plots_dir_pattern="{base_output_dir}/{replanning_variant}/{experiment_set_name}/analysis/plots/per_seed"

  echo "Plotting average travel times and summed departures per seed for parameters: replanning_variant=${replanning_variant}, beta=$beta, read_from_random=$java_seed_index, use_random_seed=$rust_seed"


  # Plot the average travel times and summed departures.
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/tt_and_sd_single_seed.py \
    "$beta" "${replanning_variant}" "$java_seed_index" "$rust_seed" "${experiment_set_name}" "${base_output_dir}" \
    "${input_tt_csv_path_pattern}" "${input_sd_csv_path_pattern}"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_per_seed_case