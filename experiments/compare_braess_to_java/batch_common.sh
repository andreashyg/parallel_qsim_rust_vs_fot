#!/bin/bash

set -uo pipefail

# Variants for the replanning strategy to use in the experiments. These simply correspond to variants that were used
# in the Java implementation of the Braess experiment, and whose output plans are read by the Rust implementation to
# simulate the same scenario.
REPLANNING_VARIANTS=(
  sel-exp1-switch-at50
  sel-exp1-switch-at80
  sel-exp10-switch-at80
)

# Parameter for time steps and vehicle size.
# One time step has length 1/beta seconds (i.e., we have beta ticks per second).
# Vehicles have pce 1/(beta^2) and length 7.5/(beta^2)
BETAS=(1 2 4 8 16)

WHICH_SEEDS_TO_AVG_OVER=("java" "rust")  # either "java" or "rust", to select over which random seeds to iterate or average over.

# these are the counters for the reruns with different random seeds that were performed in Java. Meaning that these
# values are used to read the corresponding output plans from the Java implementation of the Braess experiment.
#READ_FROM_RANDOM_VALUES=({1..1})  # only use the first random seed used in Java, because the differences there are not
# what we are interested in here, instead it's more relevant to compare different random seeds in Rust.

# random seeds used in the rust simulations
#USE_RANDOM_SEED_VALUES=({42..61})  # arbitrary range of 20 random seeds to use in the Rust simulations

SIM_OUTPUT_BASE_DIR="./../runs-svn/Abschlussarbeiten/2026/andreas-hygrell-rust-vs-fot/compare_braess_to_java"

# Collect failures so the scripts can finish all experiments/extractions and only report problems at the end.
EXPERIMENT_FAILURES=()
EXTRACTION_FAILURES=()
PLOTTING_FAILURES=()

# if this command line argument is given, the rust config will use config.overwrite_files = DeleteDirectoryIfExists
delete_output_dir_if_existing=false
# if this command line argument is given, the bash script will skip the experiment case if the output directory already
# exists.
skip_existing_output_dir=false
# other command line arguments are not supported, but could be added here in the future.

# this is a function called to read command line arguments
parse_common_args() {
  for arg in "$@"; do
    case "$arg" in
      --delete-output-dir-if-existing)
        delete_output_dir_if_existing=true
        ;;
      --skip-existing-output-dir)
        skip_existing_output_dir=true
        ;;
      *)
        echo "Unknown argument: $arg" >&2
        echo "Usage: $0 [--delete-output-dir-if-existing  --skip-existing-output-dir]" >&2
        exit 1
        ;;
    esac
  done
}

# function called to call another function (e.g. the actual run of an experiment) repeatedly for every combination of
# parameters (replanning_variant, averaging over java seeds or rust seeds, beta, read_from_random, use_random_seed).
# The function to call is passed as the first argument.
# the second parameter selects between calling the callback once for each random seed (for_which="per_seed") or
# once for all seeds but separately for different betas, (for_which="all_seeds_avgd") or once for each replanning
# variant (for_which="once_per_replanning_variant").
# The callback will be called with the parameters:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# or
#   replanning_variant, beta, use_random_seed, seeds_to_avg_over
# or
#   replanning_variant, seeds_to_avg_over
# for for_which="per_seed", for_which="all_seeds_avgd" and for_which="once_per_replanning_variant", respectively.
for_each_experiment_case() {
  local callback="$1"
  local for_which="$2"

  # Iterate over every parameter combination and delegate the work to the given callback.
  for replanning_variant in "${REPLANNING_VARIANTS[@]}"; do
    for seeds_to_avg_over in "${WHICH_SEEDS_TO_AVG_OVER[@]}"; do
      # if for_which="once_per_replanning_variant", then the callback is called once for each replanning variant, and the
      # callback is expected to handle averaging over beta and read_from_random or use_random_seed itself.
      if [ "${for_which}" = "once_per_replanning_variant" ]; then
        "$callback" "$replanning_variant" "$seeds_to_avg_over"
      # else, continue with the nested loops over beta, read_from_random or use_random_seed.
      else
        if [ "$seeds_to_avg_over" = "java" ]; then
          read_from_random_values=({1..20})  # iterate or average over all random seeds used in java
          use_random_seed_values=({42..42})  # only consider one random seed for the simulations in Rust
          for beta in "${BETAS[@]}"; do
            # Note: this is actually just one value. It will be used as a fixed seed for the rust simulations,
            # but the callback will be called separately for each read_from_random value (=java seed), or alternatively
            # (if for_which="all_seeds_avgd") the callback will be called just once, but will internally consider all
            # read_from_random values (=java seeds) and average over them
            for use_random_seed in "${use_random_seed_values[@]}"; do
              # for_which="all_seeds_avgd" means that the callback is called once for all seeds (in this case: once for
              # all read_from_random values that we want to consider, instead of an extra for loop)
              # and the callback is expected to handle the averaging over read_from_random values itself.
              if [ "${for_which}" = "all_seeds_avgd" ]; then
                "$callback" "$replanning_variant" "$beta" "$use_random_seed" "$seeds_to_avg_over"

              # for_which="per_seed" means that the callback is called once for each read_from_random value
              elif [ "${for_which}" = "per_seed" ]; then
                for read_from_random in "${read_from_random_values[@]}"; do
                  local output_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"

                  "$callback" "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed" "$seeds_to_avg_over" "$output_dir"
                done
              else
                echo "Unknown for_which value: $for_which" >&2
                exit 1
              fi
            done
          done
        elif [ "$seeds_to_avg_over" = "rust" ]; then
          read_from_random_values=({1..1})  # only use the first random seed used in Java
          use_random_seed_values=({42..61})  # arbitrary range of 20 random seeds to use in the Rust simulations
          for beta in "${BETAS[@]}"; do
            # Note: this is actually just one value. It will be used as a fixed seed to read from the Java output plans,
            # but the callback will be called separately for each use_random_seed value (=rust seed), or alternatively
            # (if for_which="all_seeds_avgd") the callback will be called just once, but will internally consider all
            # use_random_seed values (=rust seeds) and average over them
            for read_from_random in "${read_from_random_values[@]}"; do
              # for_which="all_seeds_avgd" means that the callback is called once per seed (in this case: per
              # use_random_seed value that we want to consider, instead of an extra for loop)
              # and the callback is expected to handle the averaging over use_random_seed values (rust seeds) itself.
              if [ "${for_which}" = "all_seeds_avgd" ]; then
                "$callback" "$replanning_variant" "$beta" "$read_from_random" "$seeds_to_avg_over"

              # for_which="per_seed" means that the callback is called once for each use_random_seed value
              elif [ "${for_which}" = "per_seed" ]; then
                for use_random_seed in "${use_random_seed_values[@]}"; do
                  local output_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"

                  "$callback" "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed" "$seeds_to_avg_over" "$output_dir"
                done
              else
                echo "Unknown for_which value: $for_which" >&2
                exit 1
              fi
            done
          done
        else
          echo "Unknown seed type: $seeds_to_avg_over" >&2
          exit 1
        fi

      fi
    done
  done
}

# function to called to record a failure in either the experiment run or the extraction or plotting run.
# Called with the arguments:
#   kind: either "experiment" or "extraction" or "plotting"
# and
#   replanning_variant, beta, read_from_random, output_dir, use_random_seed
# Will append a string describing the failure to the corresponding array (EXPERIMENT_FAILURES or EXTRACTION_FAILURES
# or PLOTTING_FAILURES).
record_failure() {
  local kind="$1"
  local replanning_variant="$2"
  local beta="$3"
  local read_from_random="$4"
  local use_random_seed="$5"

  case "$kind" in
    experiment)
      EXPERIMENT_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
      ;;
    extraction)
      EXTRACTION_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
      ;;
    plotting)
      PLOTTING_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
      ;;
    *)
      echo "Unknown failure kind: $kind" >&2
      exit 1
      ;;
  esac
}

# function to print a summary of all failures recorded during the batch run. Will print the number of failures and the
# details of each failure.
print_failure_summary() {
  if [ "${#EXPERIMENT_FAILURES[@]}" -eq 0 ] && [ "${#EXTRACTION_FAILURES[@]}" -eq 0 ] && [ "${#PLOTTING_FAILURES[@]}" -eq 0 ]; then
    echo "No failures recorded."
    return 0
  fi

  echo "Failure summary:"

  if [ "${#EXPERIMENT_FAILURES[@]}" -gt 0 ]; then
    echo "  Experiment failures (${#EXPERIMENT_FAILURES[@]}):"
    for failure in "${EXPERIMENT_FAILURES[@]}"; do
      echo "    - $failure"
    done
  fi

  if [ "${#EXTRACTION_FAILURES[@]}" -gt 0 ]; then
    echo "  Extraction failures (${#EXTRACTION_FAILURES[@]}):"
    for failure in "${EXTRACTION_FAILURES[@]}"; do
      echo "    - $failure"
    done
  fi

  if [ "${#PLOTTING_FAILURES[@]}" -gt 0 ]; then
    echo "  Plotting failures (${#PLOTTING_FAILURES[@]}):"
    for failure in "${PLOTTING_FAILURES[@]}"; do
      echo "    - $failure"
    done
  fi
}

# function to run a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# Will run the Rust simulation with the given parameters and write the output to the given output directory.
# If the simulation fails, it will record the failure and continue with the next case
run_experiment_case() {
  local replanning_variant="$1"
  local beta="$2"
  local read_from_random="$3"
  local use_random_seed="$4"
  local seeds_to_avg_over="$5"
  local output_dir="$6"

  if [ -d "$output_dir" ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping simulation due to existing output directory: $output_dir"
    return 0
  fi


  echo "Running rust simulation with parameters: replanning_variant=${replanning_variant}, beta=$beta, read_from_random=$read_from_random, use_random_seed=$use_random_seed"

  echo "Output directory: $output_dir"

  local delete_output_dir_arg=()
  if [ "$delete_output_dir_if_existing" = true ]; then
    delete_output_dir_arg=(--delete-output-dir-if-existing)
  fi

  # Run the Rust simulation; failures are reported but do not stop the batch.
  if ! cargo run --release --bin run_braess_from_java -- \
    --beta "$beta" \
    --read-from-random "$read_from_random" \
    --use-random-seed "$use_random_seed" \
    --replanning-variant "$replanning_variant" \
    --output-dir "$output_dir" \
    "${delete_output_dir_arg[@]}"
  then
    echo "Experiment failed, continuing with next case." >&2
    record_failure experiment "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed"
    return 1
  fi

  return 0
}

# function to run the travel time & summed departures extraction for a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# Will read the events written by the simulation in the given output directory and write the average travel times per
# route to a CSV file in the analysis subdirectory of the output root directory.
extract_travel_time_sum_dep_case() {
  local replanning_variant="$1"
  local beta="$2"
  local read_from_random="$3"
  local use_random_seed="$4"
  local seeds_to_avg_over="$5"
  local output_dir="$6"

  local extracted_data_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/extracted_data"


  if [ -d extracted_data_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping travel time/summed departures extraction because the output directory already exists: $extracted_data_dir"
    return 0
  fi

  local tt_csv_path="${extracted_data_dir}/average_route_tts_per_deptime_beta${beta}_read_from_random_${read_from_random}_use_random_seed_${use_random_seed}.csv"
  local sd_csv_path="${extracted_data_dir}/summed_deps_per_time_beta${beta}_read_from_random_${read_from_random}_use_random_seed_${use_random_seed}.csv"


  echo "Extracting average travel times and summed departures for parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random=$read_from_random"

  local input_file_stem="${output_dir}/events/events"

  echo "writing into $tt_csv_path and $sd_csv_path"

  # Extract travel times and summed departures from the events written by the simulation.
  if ! cargo run --release --bin event_data_extractor -- \
    --input-file-stem "$input_file_stem" \
    --input-file-format "binpb" \
    --tt-csv-path "$tt_csv_path" \
    --sd-csv-path "$sd_csv_path" \
    --num-parts 1 \
    --link-to-path-map-name "braess" \
    --id-store-path "${output_dir}/output_ids.binpb" \
    --beta "$beta"
  then
    echo "Travel-time/summed departures extraction failed, continuing with next case." >&2
    record_failure extraction "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed"
    return 1
  fi

  return 0
}

plot_per_seed_case() {
  local replanning_variant="$1"
  local beta="$2"
  local read_from_random="$3"
  local use_random_seed="$4"
  local seeds_to_avg_over="$5"
  local output_dir="$6"

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/per_seed"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting per seed because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures per seed for parameters: replanning_variant=${replanning_variant}, beta=$beta, read_from_random=$read_from_random, use_random_seed=$use_random_seed, seeds_to_avg_over=$seeds_to_avg_over"


  # Plot the average travel times and summed departures.
  if ! python python_plotting/braess/tt_and_sd_single_seed.py "$beta" "${replanning_variant}" "$read_from_random" "$use_random_seed" "$seeds_to_avg_over" "$output_plots_dir"
  then
    echo "Plotting failed, continuing with next case." >&2
    record_failure plotting "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed"
    return 1
  fi

  return 0
}

plot_avg_over_seeds_case() {
  local replanning_variant="$1"
  local beta="$2"
  local fixed_seed="$3" # this can be either a use_random_seed value (if seeds_to_avg_over=java) or a read_from_random value (if seeds_to_avg_over=rust)
  local seeds_to_avg_over="$4"

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/avg_over_${seeds_to_avg_over}_seeds"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting avg over ${seeds_to_avg_over} seeds because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures averaged over ${seeds_to_avg_over} seeds for parameters: replanning_variant=${replanning_variant}, beta=$beta, fixed_seed=$fixed_seed, seeds_to_avg_over=$seeds_to_avg_over"

  # Plot the average travel times and summed departures averaged over seeds.
  if ! python python_plotting/braess/tt_and_sd_avg_over_seeds.py "$beta" "${replanning_variant}" "$seeds_to_avg_over" "$fixed_seed" "$output_plots_dir"
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

plot_once_per_replanning_variant_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"  # either "java" or "rust"

  local output_plots_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/plots/deviations/avg_over_${seeds_to_avg_over}_seeds"

  if [ -d output_plots_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping plotting once per replanning variant because output plots directory already exists: $output_plots_dir"
    return 0
  fi

  echo "Plotting average travel times and summed departures once per replanning variant for parameters: replanning_variant=${replanning_variant}, seeds_to_avg_over=${seeds_to_avg_over}"

  # Plot the average travel times and summed departures once per replanning variant.
  if ! python python_plotting/braess/tt_and_sd_dev_over_beta.py "${replanning_variant}" "$seeds_to_avg_over" "$output_plots_dir"
  then
    echo "Plotting failed, continuing with next case." >&2
    if [ "$seeds_to_avg_over" = "java" ]; then
      record_failure plotting "$replanning_variant" "all_betas" "avg_over_java_seeds" "$fixed_seed"
    elif [ "$seeds_to_avg_over" = "rust" ]; then
      record_failure plotting "$replanning_variant" "all_betas" "$fixed_seed" "avg_over_rust_seeds"
    else
      echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
      exit 1
    fi
    return 1
  fi

  return 0
  }

# function to run a single experiment case and then extract the travel times from the events written by the simulation.
run_and_extract_case() {
  # Only extract travel times if the simulation run finished successfully.
  if run_experiment_case "$@"; then
    extract_travel_time_sum_dep_case "$@"
  else
    echo "Skipping travel time/summed departures extraction because the experiment failed." >&2
  fi

  return 0
}

run_and_extract_and_plot_per_seed_case() {
  # Only extract travel times if the simulation run finished successfully.
  if run_and_extract_case "$@"; then
    plot_per_seed_case "$@"
  else
    echo "Skipping plotting because the experiment or travel time/summed departures extraction failed." >&2
  fi

  return 0
}