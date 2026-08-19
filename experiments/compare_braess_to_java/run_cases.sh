#!/bin/bash

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
                "$callback" "$replanning_variant" "$seeds_to_avg_over" "$beta"

              # for_which="per_seed" means that the callback is called once for each read_from_random value
              elif [ "${for_which}" = "per_seed" ]; then
                for read_from_random in "${read_from_random_values[@]}"; do
#                  local output_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"

                  "$callback" "$replanning_variant" "$seeds_to_avg_over" "$beta" "$read_from_random"
                done
              else
                echo "Unknown for_which value: $for_which" >&2
                exit 1
              fi
            done
          done
        elif [ "$seeds_to_avg_over" = "rust" ]; then
          read_from_random_values=({1..1})  # only use the first random seed used in Java
          use_random_seed_values=({1..20})  # changed to also be 1..20 now, since we get the actual seeds automatically
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
                "$callback" "$seeds_to_avg_over" "$replanning_variant" "$beta"

              # for_which="per_seed" means that the callback is called once for each use_random_seed value
              elif [ "${for_which}" = "per_seed" ]; then
                # run the callback once without a use_random_seed value and instead read_original_java=true, which means
                # that the original data will be processed/plotted (depending on the callback) for comparison with the
                # reruns with different random seeds in Rust.
#                local output_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_original_java_data"

                "$callback" "$replanning_variant" "$seeds_to_avg_over" "$beta" "$read_from_random"

                for use_random_seed in "${use_random_seed_values[@]}"; do
#                  local output_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"

                  "$callback" "$replanning_variant" "$seeds_to_avg_over" "$beta" "$use_random_seed"
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
