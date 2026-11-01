#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

plot_per_seed_case() {
  local beta="$1"
  local java_seed_index="$2"
  local rust_seed="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local input_tt_csv_path_pattern="$7"
  local input_sd_csv_path_pattern="$8"
  local sd_ybotlim="$9"
  local sd_ytoplim="${10}"
  local tt_ybotlim="${11}"
  local tt_ytoplim="${12}"


  echo "Plotting average travel times and summed departures per seed for parameters: replanning_variant=deprecated, beta=$beta, read_from_random=$java_seed_index, use_random_seed=$rust_seed"


  # Plot the average travel times and summed departures.
  if ! python python_plotting/braess/tt_and_sd_single_seed.py \
    "$beta" "$java_seed_index" "$rust_seed" "${experiment_set_name}" "${base_output_dir}" \
    "${input_tt_csv_path_pattern}" "${input_sd_csv_path_pattern}" "${global_config_path}" \
    "$sd_ybotlim" "$sd_ytoplim" "$tt_ybotlim" "$tt_ytoplim"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "deprecated" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_per_seed_case