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
  local _experiment_output_dir_pattern="$7"  # unused in this module
  local _delete_output_dir_if_existing="$8"  # unused in this module
  local output_tt_plot_path_pattern="$9"
  local output_sd_plot_path_pattern="${10}"
  local input_tt_csv_path_pattern="${11}"
  local input_sd_csv_path_pattern="${12}"
  local seeds_to_use_array_string="${13}"


  read -a seeds_to_use <<< "$seeds_to_use_array_string"
#  if [ "${seeds_to_avg_over}" = "java" ]; then
#    local fixed_seed="${RUST_SEED_WHEN_FIXED}"
#  elif [ "${seeds_to_avg_over}" = "rust" ]; then
#    local fixed_seed="${JAVA_SEED_INDEX_WHEN_FIXED}"
#  else
#    echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
#    exit 1
#  fi
#
#  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/avg_over_${seeds_to_avg_over}_seeds"
#
#  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
#    echo "Skipping plotting avg over ${seeds_to_avg_over} seeds because output plots directory already exists: $output_plots_dir"
#    return 0
#  fi
  printf "Seeds to use for averaging: %s\n" "${seeds_to_use[*]}"
  for arg in "${seeds_to_use[@]}"; do
    if [[ ! "$arg" =~ ^[0-9]+$ ]]; then
      echo "Invalid seed value: $arg. All seeds must be integers." >&2
      record_failure plotting "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
      return 1
    else
        echo "Valid seed value: $arg"
    fi
  done

  echo "Plotting average travel times and summed departures averaged over seeds for parameters: replanning_variant=${replanning_variant}, beta=$beta, java_seed_index=$java_seed_index, rust_seed=$rust_seed with seeds to use = ${seeds_to_use[*]}"

  # Plot the average travel times and summed departures averaged over seeds.
  if ! ~/miniforge3/envs/rust-vs-fot-plots/bin/python3 python_plotting/braess/tt_and_sd_avg_over_seeds.py "$beta" "${replanning_variant}" "$experiment_set_name" "$java_seed_index" "$rust_seed" "$base_output_dir" "$output_tt_plot_path_pattern" "$output_sd_plot_path_pattern" "$input_tt_csv_path_pattern" "$input_sd_csv_path_pattern" "${seeds_to_use[@]}"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$java_seed_index" "$rust_seed"
    return 1
  fi

  return 0
}

export -f plot_avgd_over_seeds_case