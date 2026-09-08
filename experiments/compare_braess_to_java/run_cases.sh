#!/bin/bash

# function to run a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# Will run the Rust simulation with the given parameters and write the output to the given output directory.
# If the simulation fails, it will record the failure and continue with the next case
run_experiment_case() {
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

  local output_dir
  output_dir=$(get_simulation_output_directory "$replanning_variant" "$seeds_to_avg_over" "$beta" "$read_from_random" "$use_random_seed")

  if [ -d "$output_dir" ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping simulation due to existing output directory: $output_dir"
    return 0
  fi


  echo "Running rust simulation with parameters: replanning_variant=${replanning_variant}, seeds_to_avg_over=${seeds_to_avg_over}, beta=${beta}, read_from_random=${read_from_random}, use_random_seed=${use_random_seed}"

  echo "Output directory: $output_dir"

  local delete_output_dir_arg=()
  if [ "$delete_output_dir_if_existing" = true ]; then
    delete_output_dir_arg=(--delete-output-dir-if-existing)
  fi

  # Run the Rust simulation; failures are reported but do not stop the batch.
  if ! cargo run --release --bin run_single_braess_iter_from_java_output -- \
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
