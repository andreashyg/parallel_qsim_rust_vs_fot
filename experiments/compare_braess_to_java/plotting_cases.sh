#!/bin/bash

plot_per_seed_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local beta="$3"
  local seed_index="$4"

  # this will set read_from_random to the java seed index based on whether we are averaging over java seeds or rust
  # seeds.
  # In the former case, gets the corresponding java seed index from the array of java seeds to iterate over,
  # in the latter case, gets the fixed java seed index.
  local read_from_random
  read_from_random=$(get_java_seed_from_index "$seeds_to_avg_over" "$seed_index")
  # Same thing for rust seeds
  local use_random_seed
  use_random_seed=$(get_rust_seed_from_index "$seeds_to_avg_over" "$seed_index")

  # directory where the plots will be written to.
  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/per_seed"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting per seed because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures per seed for parameters: replanning_variant=${replanning_variant}, beta=$beta, read_from_random=$read_from_random, use_random_seed=$use_random_seed, seeds_to_avg_over=$seeds_to_avg_over"


  # Plot the average travel times and summed departures.
  if ! python python_plotting/braess/tt_and_sd_single_seed.py "$beta" "${replanning_variant}" "$read_from_random" "$use_random_seed" "$seeds_to_avg_over" "false" "$output_plots_dir"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed"
    return 1
  fi

  return 0
}

plot_original_java_data_per_seed_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local beta="$3"
  local seed_index="$4"

  if [ "${seeds_to_avg_over}" != "java" ]; then
    echo "plot_original_java_data_per_seed_case is only applicable when seeds_to_avg_over is 'java', but got: $seeds_to_avg_over" >&2
    return 0
  fi

  # this will set read_from_random to the java seed index based on whether we are averaging over java seeds or rust
  # seeds.
  # In the former case, gets the corresponding java seed index from the array of java seeds to iterate over,
  # in the latter case, gets the fixed java seed index.
  local read_from_random
  read_from_random=$(get_java_seed_from_index "$seeds_to_avg_over" "$seed_index")
#  # Same thing for rust seeds
#  local use_random_seed
#  use_random_seed=$(get_rust_seed_from_index "$seeds_to_avg_over" "$seed_index")

  # directory where the plots will be written to.
  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/recreating_java_results/analysis/plots/per_seed"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting per seed because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures per seed for parameters: replanning_variant=${replanning_variant}, beta=$beta, read_from_random=$read_from_random for ORIGINAL JAVA DATA"

  # Plot the average travel times and summed departures.
  if ! python python_plotting/braess/tt_and_sd_single_seed.py "$beta" "${replanning_variant}" "$read_from_random" "None" "$seeds_to_avg_over" "true" "$output_plots_dir"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$read_from_random" "reading_original_java_data"
    return 1
  fi

  return 0
}

plot_avg_over_seeds_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local beta="$3"
  #local fixed_seed="$4" # this can be either a use_random_seed value (if seeds_to_avg_over=java) or a read_from_random value (if seeds_to_avg_over=rust)

  if [ "${seeds_to_avg_over}" = "java" ]; then
    local fixed_seed="${RUST_SEED_WHEN_FIXED}"
  elif [ "${seeds_to_avg_over}" = "rust" ]; then
    local fixed_seed="${JAVA_SEED_INDEX_WHEN_FIXED}"
  else
    echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
    exit 1
  fi

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/avg_over_${seeds_to_avg_over}_seeds"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting avg over ${seeds_to_avg_over} seeds because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures averaged over ${seeds_to_avg_over} seeds for parameters: replanning_variant=${replanning_variant}, beta=$beta, fixed_seed=$fixed_seed, seeds_to_avg_over=$seeds_to_avg_over"

  # Plot the average travel times and summed departures averaged over seeds.
  if ! python python_plotting/braess/tt_and_sd_avg_over_seeds.py "$beta" "${replanning_variant}" "$seeds_to_avg_over" "$fixed_seed" "$output_plots_dir" "false"
  then
    echo "Plotting failed, continuing with next case." >&2
    if [ "$seeds_to_avg_over" = "java" ]; then
      record_failure plotting "$replanning_variant" "$beta" "avg_over_java_seeds" "$fixed_seed"
    elif [ "$seeds_to_avg_over" = "rust" ]; then
      record_failure plotting "$replanning_variant" "$beta" "$fixed_seed" "avg_over_rust_seeds"
    else
      echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
      exit 1
    fi
    return 1
  fi

  return 0
}

plot_original_java_data_avg_over_seeds_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local beta="$3"

  if [ "${seeds_to_avg_over}" = "rust" ]; then
    # skip, since we are plotting original java data, it doesn't make sense to average over rust seeds.
    return 0
  elif [ "${seeds_to_avg_over}" != "java" ]; then
    echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
    exit 1
  fi

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/recreating_java_results/analysis/plots/avg_over_${seeds_to_avg_over}_seeds"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting avg over ${seeds_to_avg_over} seeds because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures averaged over ${seeds_to_avg_over} seeds for parameters: replanning_variant=${replanning_variant}, beta=$beta for ORIGINAL JAVA DATA"

  # Plot the average travel times and summed departures averaged over seeds.
  if ! python python_plotting/braess/tt_and_sd_avg_over_seeds.py "$beta" "${replanning_variant}" "$seeds_to_avg_over" "None" "$output_plots_dir" "true"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "avg_over_java_seeds" "ORIGINAL JAVA DATA"
    return 1
  fi

  return 0
}

plot_once_per_replanning_variant_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"  # either "java" or "rust"

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/deviations/"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting once per replanning variant because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures once per replanning variant for parameters: replanning_variant=${replanning_variant}, seeds_to_avg_over=${seeds_to_avg_over}"

  # Plot the average travel times and summed departures once per replanning variant.
  if ! python python_plotting/braess/tt_and_sd_dev_over_beta.py "${replanning_variant}" "$seeds_to_avg_over" "$output_plots_dir" "false"
  then
    echo "Plotting failed, continuing with next case." >&2
    if [ "$seeds_to_avg_over" = "java" ]; then
      record_failure plotting "$replanning_variant" "all_betas" "avg_over_java_seeds" "default fixed seed 42"
    elif [ "$seeds_to_avg_over" = "rust" ]; then
      record_failure plotting "$replanning_variant" "all_betas" "default fixed seed 1" "avg_over_rust_seeds"
    else
      echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
      exit 1
    fi
    return 1
  fi

  return 0
}

plot_original_java_data_once_per_replanning_variant_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"  # either "java" or "rust"

  if [ "${seeds_to_avg_over}" = "rust" ]; then
    # skip, since we are plotting original java data, it doesn't make sense to average over rust seeds.
    return 0
  elif [ "${seeds_to_avg_over}" != "java" ]; then
    echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
    exit 1
  fi

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/recreating_java_results/analysis/plots/deviations/"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting once per replanning variant because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures once per replanning variant for parameters: replanning_variant=${replanning_variant} for ORIGINAL JAVA DATA"

  # Plot the average travel times and summed departures once per replanning variant.
  if ! python python_plotting/braess/tt_and_sd_dev_over_beta.py "${replanning_variant}" "$seeds_to_avg_over" "$output_plots_dir" "true"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "all_betas" "avg_over_java_seeds" "ORIGINAL JAVA DATA"
    return 1
  fi

  return 0
}
