#!/bin/bash

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

read_config_file_entry() {
  local -n var_to_change="$1"
  local key="$2"
  mapfile -t var_to_change < <(yq -r ".${key} | if type == \"array\" then .[] else . end" "$SCRIPT_DIR/../config.yaml")
}

# Variants for the replanning strategy to use in the experiments. These simply correspond to variants that were used
# in the Java implementation of the Braess experiment, and whose output plans are read by the Rust implementation to
# simulate the same scenario.
read_config_file_entry REPLANNING_VARIANTS replanning_variants

# Parameter for time steps and vehicle size.
# One time step has length 1/beta seconds (i.e., we have beta ticks per second).
# Vehicles have pce 1/(beta^2) and length 7.5/(beta^2)
read_config_file_entry BETAS betas

# either "java" or "rust", to select over which random seeds to iterate or average over.
WHICH_SEEDS_TO_AVG_OVER=(
  "java"
  "rust"
  )

# when fixing a rust seed and varying java (e.g.: plot results based on java runs using different seeds there, but
# all just run with a single rust seed), use this seed
read_config_file_entry RUST_SEED_WHEN_FIXED rust_seed_when_fixed

# when fixing a java seed (while varying rust seeds), use this index (the actual seed differs)
read_config_file_entry JAVA_SEED_INDEX_WHEN_FIXED java_seed_index_when_fixed

# when varying rust seeds, use these
read_config_file_entry RUST_SEEDS_TO_ITERATE_OVER rust_seeds_to_iterate_over

# when varying java seeds, use these indices (the actual seeds differ)
read_config_file_entry JAVA_SEED_INDICES_TO_ITERATE_OVER java_seed_indices_to_iterate_over

# Base directory where the simulation output directories will be created.
read_config_file_entry SIM_OUTPUT_BASE_DIR sim_output_base_dir

# Collect failures so the scripts can finish all experiments/extractions and only report problems at the end.
EXPERIMENT_FAILURES=()
EXTRACTION_FAILURES=()
PLOTTING_FAILURES=()
DUMMY_COORDINATE_ADDING_FAILURES=()
REFORMATTING_FAILURES=()

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

get_simulation_output_directory() {
  local replanning_variant=$1
  local seeds_to_avg_over=$2
  local beta=$3
  local read_from_random=$4
  local use_random_seed=$5

  echo "${SIM_OUTPUT_BASE_DIR}${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"
}

get_java_seed_from_index() {
  local seeds_to_avg_over="$1"
  local seed_index="$2"

  if [ "$seeds_to_avg_over" = "java" ]; then
    echo "${JAVA_SEED_INDICES_TO_ITERATE_OVER[$seed_index]}"
  elif [ "$seeds_to_avg_over" = "rust" ]; then
    echo "$JAVA_SEED_INDEX_WHEN_FIXED"
  else
    echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
    exit 1
  fi
}

get_rust_seed_from_index() {
  local seeds_to_avg_over="$1"
  local seed_index="$2"

  if [ "$seeds_to_avg_over" = "rust" ]; then
    echo "${RUST_SEEDS_TO_ITERATE_OVER[$seed_index]}"
  elif [ "$seeds_to_avg_over" = "java" ]; then
    echo "$RUST_SEED_WHEN_FIXED"
  else
    echo "Unknown seeds_to_avg_over value: $seeds_to_avg_over" >&2
    exit 1
  fi
}

get_original_java_replanning_folder_string() {
  local replanning_variant="$1"

  if [ "${replanning_variant}" = "sel-exp1-switch-at50" ]; then
    echo "2026-05-8-12-16-8_500it_reRouteProba0.1until0.5it_selExpBeta1proba0.9_msaFrom0.5it"
  elif [ "${replanning_variant}" = "sel-exp1-switch-at80" ]; then
    echo "2026-05-10-10-2-21_500it_reRouteProba0.1until0.8it_selExpBeta1proba0.9_msaFrom0.8it"
  elif [ "${replanning_variant}" = "sel-exp10-switch-at80" ]; then
    echo "2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it"
  else
    echo "unknown replanning_variant value: $replanning_variant" >&2
    return 1
  fi
}
# function that iterates through every combination
# of replanning variant and "seed to vary", and calls the callback
# for those two parameters, plus any additional parameters passed to this function.
run_for_each_replvar_variedseed_combo() {
  local callback=$1
  shift  # consumes the first argument
  # this means that "$@" is now all arguments after callback

  # Iterate over every parameter combination and delegate the work to the given callback.
  for replanning_variant in "${REPLANNING_VARIANTS[@]}"; do
    for seeds_to_avg_over in "${WHICH_SEEDS_TO_AVG_OVER[@]}"; do
      "$callback" "$replanning_variant" "$seeds_to_avg_over" "$@"
    done
  done
}

run_all_betas_for_replvar_variedseed_combo() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local actual_callback="$3"

  shift 3

  for beta in "${BETAS[@]}"; do
    "$actual_callback" "$replanning_variant" "$seeds_to_avg_over" "$beta" "$@"
  done
}

run_for_each_replvar_variedseed_beta_combo() {
  local actual_callback=$1
  shift

  # This will:
  # - iterate over every replanning_variant and seeds_to_avg_over combination
  # - for each combination, iterate over every beta and call actual_callback with replanning_variant, seeds_to_avg_over,
  #   beta, and any additional parameters passed to this function.
  run_for_each_replvar_variedseed_combo run_all_betas_for_replvar_variedseed_combo "$actual_callback" "$@"


#  for beta in "${BETAS[@]}"; do
#    run_for_each_replvar_variedseed_combo "$actual_callback" "$beta" "$@"
#  done
}

run_all_seeds_for_replvar_variedseed_beta_combo() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local beta="$3"
  local actual_callback="$4"

  shift 4

  for seed_index in {0..19}; do
    "$actual_callback" "$replanning_variant" "$seeds_to_avg_over" "$beta" "$seed_index" "$@"
  done
}

run_for_each_replvar_variedseed_beta_seed_combo() {
  local actual_callback=$1
  shift

  # This will:
  # - iterate over every replanning_variant, seeds_to_avg_over and beta combination
  # - for each combination, iterate over every seed_index and call actual_callback with replanning_variant,
  # seeds_to_avg_over, beta, seed_index and any other parameters
  run_for_each_replvar_variedseed_beta_combo run_all_seeds_for_replvar_variedseed_beta_combo "$actual_callback" "$@"
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
    dummy_coordinate_adding)
      DUMMY_COORDINATE_ADDING_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
      ;;
    reformatting)
      REFORMATTING_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
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

  if [ "${#DUMMY_COORDINATE_ADDING_FAILURES[@]}" -gt 0 ]; then
    echo "  Dummy coordinate adding failures (${#DUMMY_COORDINATE_ADDING_FAILURES[@]}):"
    for failure in "${DUMMY_COORDINATE_ADDING_FAILURES[@]}"; do
      echo "    - $failure"
    done
  fi

  if [ "${#REFORMATTING_FAILURES[@]}" -gt 0 ]; then
    echo "  Reformatting failures (${#REFORMATTING_FAILURES[@]}):"
    for failure in "${REFORMATTING_FAILURES[@]}"; do
      echo "    - $failure"
    done
  fi
}

__BATCH_COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${__BATCH_COMMON_DIR}/run_cases.sh"
source "${__BATCH_COMMON_DIR}/extraction_cases.sh"
source "${__BATCH_COMMON_DIR}/plotting_cases.sh"