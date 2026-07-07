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

# these are the counters for the reruns with different random seeds that were performed in Java. Meaning that these
# values are used to read the corresponding output plans from the Java implementation of the Braess experiment.
READ_FROM_RANDOM_VALUES=({1..1})  # only use the first random seed used in Java, because the differences there are not
# what we are interested in here, instead it's more relevant to compare different random seeds in Rust.

# random seeds used in the rust simulations
USE_RANDOM_SEED_VALUES=({42..61})  # arbitrary range of 20 random seeds to use in the Rust simulations

OUTPUT_BASE_DIR="./../runs-svn/Abschlussarbeiten/2026/andreas-hygrell-rust-vs-fot/compare_braess_to_java"

# Collect failures so the scripts can finish all experiments/extractions and only report problems at the end.
EXPERIMENT_FAILURES=()
EXTRACTION_FAILURES=()

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

# function called to call another function (the actual run of an experiment) repeatedly for every combination of
# parameters (replanning_variant, beta, read_from_random). The function to call is passed as the first argument.
# It will be called with the parameters:
#   replanning_variant, beta, read_from_random, use_random_seed, output_root_dir, output_dir
# accordingly.
for_each_experiment_case() {
  local callback="$1"

  # Iterate over every experiment combination and delegate the work to the given callback.
  for replanning_variant in "${REPLANNING_VARIANTS[@]}"; do
    local output_root_dir="${OUTPUT_BASE_DIR}/${replanning_variant}"
    for beta in "${BETAS[@]}"; do
      for read_from_random in "${READ_FROM_RANDOM_VALUES[@]}"; do
        for use_random_seed in "${USE_RANDOM_SEED_VALUES[@]}"; do
          local output_dir="${output_root_dir}/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"

          # if the output directory already exists and the corresponding cla was given, skip this experiment
          skip=false
          if [ -d "$output_dir" ] && [ "${skip_existing_output_dir}" = "true" ]; then
            skip=true
            echo "Skipping due to existing output directory: $output_dir"
          fi
          "$callback" "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed" "$output_root_dir" "$output_dir" "$skip"
        done
      done
    done
  done
}

# function to called to record a failure in either the experiment run or the extraction run.
# Called with the arguments:
#   kind: either "experiment" or "extraction"
# and
#   replanning_variant, beta, read_from_random, output_dir, use_random_seed
# Will append a string describing the failure to the corresponding array (EXPERIMENT_FAILURES or EXTRACTION_FAILURES).
record_failure() {
  local kind="$1"
  local replanning_variant="$2"
  local beta="$3"
  local read_from_random="$4"
  local output_dir="$5"
  local use_random_seed="$6"

  case "$kind" in
    experiment)
      EXPERIMENT_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} dir=${output_dir} use_random_seed=${use_random_seed}")
      ;;
    extraction)
      EXTRACTION_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} dir=${output_dir} use_random_seed=${use_random_seed}")
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
  if [ "${#EXPERIMENT_FAILURES[@]}" -eq 0 ] && [ "${#EXTRACTION_FAILURES[@]}" -eq 0 ]; then
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
}

# function to run a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, output_root_dir, output_dir
# Will run the Rust simulation with the given parameters and write the output to the given output directory.
# If the simulation fails, it will record the failure and continue with the next case
run_experiment_case() {
  local replanning_variant="$1"
  local beta="$2"
  local read_from_random="$3"
  local use_random_seed="$4"
  local _output_root_dir="$5"
  local output_dir="$6"
  local skip="$7"

  if [ "$skip" = "true" ]; then
    return 0
  fi


  echo "Running rust simulation with parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random=$read_from_random, use_random_seed=$use_random_seed"

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
    record_failure experiment "$replanning_variant" "$beta" "$read_from_random" "$output_dir" "$use_random_seed"
    return 1
  fi

  return 0
}

# function to run the travel time & summed departures extraction for a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, output_root_dir, output_dir, skip
# Will read the events written by the simulation in the given output directory and write the average travel times per
# route to a CSV file in the analysis subdirectory of the output root directory.
extract_travel_time_sum_dep_case() {
  local replanning_variant="$1"
  local beta="$2"
  local read_from_random="$3"
  local use_random_seed="$4"
  local output_root_dir="$5"
  local output_dir="$6"
  local skip="$7"

  if [ "$skip" = "true" ]; then
    return 0
  fi


  echo "Extracting average travel times and summed departures for parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random=$read_from_random"

  local input_file_stem="${output_dir}/events/events"
  local tt_csv_path="${output_root_dir}/analysis/average_route_tts_per_deptime_beta${beta}_read_from_random_${read_from_random}_use_random_seed_${use_random_seed}.csv"
  local sd_csv_path="${output_root_dir}/analysis/summed_deps_per_time_beta${beta}_read_from_random_${read_from_random}_use_random_seed_${use_random_seed}.csv"

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
    record_failure extraction "$replanning_variant" "$beta" "$read_from_random" "$output_dir" "$use_random_seed"
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
