#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

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

# function to run a single experiment case. Called with the arguments:
#   replanning_variant, beta, java_seed_index, rust_seed, output_dir, delete_output_dir_if_existing, skip_existing_output_dir
# Will run the Rust simulation with the given parameters and write the output to the given output directory.
# If the simulation fails, it will record the failure and continue with the next case
run_java_single_iter_case() {
  local replanning_variant="$1"
  local beta="$2"
  local java_seed_index="$3"
  local use_this_seed_for_last_iter="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local experiment_output_dir_pattern="$7"
  local delete_output_dir_if_existing="$8"
  local config_file_pattern="$9"
  local base_dir_to_read_from="${10}"


#  local output_dir
#  output_dir=$(get_simulation_output_directory "$replanning_variant" "$seeds_to_avg_over" "$beta" "$read_from_random" "$use_random_seed")

  local original_java_replanning_variant_string
  original_java_replanning_variant_string=$(get_original_java_replanning_folder_string "$replanning_variant")

#  local config_file="${ORIGINAL_DIR_TO_READ_FROM}/${original_java_replanning_variant_string}/beta${beta}/random${read_from_random}/beta${beta}random${read_from_random}.output_config.xml"
  local config_file
  # replace {base_dir_to_read_from}, {original_java_replanning_variant_string}, {beta}, {read_from_random} in config_file_pattern with the corresponding values
  config_file=${config_file_pattern//\{base_dir_to_read_from\}/$base_dir_to_read_from}
  config_file=${config_file//\{replanning_variant\}/$original_java_replanning_variant_string}
  config_file=${config_file//\{beta\}/$beta}
  config_file=${config_file//\{read_from_random\}/$java_seed_index}

  local experiment_output_dir
  # replace {base_output_dir}, {replanning_variant}, {experiment_set_name}, {beta}, {java_seed_index}, {use_this_seed_for_last_iter} in experiment_output_dir_pattern with the corresponding values
  experiment_output_dir=${experiment_output_dir_pattern//\{base_output_dir\}/$base_output_dir}
  experiment_output_dir=${experiment_output_dir//\{replanning_variant\}/$replanning_variant}
  experiment_output_dir=${experiment_output_dir//\{experiment_set_name\}/$experiment_set_name}
  experiment_output_dir=${experiment_output_dir//\{beta\}/$beta}
  experiment_output_dir=${experiment_output_dir//\{read_from_random\}/$java_seed_index}
  experiment_output_dir=${experiment_output_dir//\{use_random_seed\}/$use_this_seed_for_last_iter}


#  local config_file="${base_dir_to_read_from}/${original_java_replanning_variant_string}/beta${beta}/random${read_from_random}/beta${beta}random${read_from_random}.output_config.xml"

  java_args=(
    run
    --config "$config_file"
    --existingRunsDir "$base_dir_to_read_from"
    --baseOutputDir "$base_output_dir"
    --replanningVariant "$replanning_variant"
    --beta "$beta"
    --readFromRandom "$java_seed_index"
    --useRandom "$use_this_seed_for_last_iter"
    --outputDir "$experiment_output_dir"
    "${delete_output_dir_arg[@]}"
  )

  # Array -> sauber gequoteter String für -Dexec.args
  printf -v exec_args '%q ' "${java_args[@]}"
  exec_args=${exec_args% }

#  JAVA_HOME=/home/andreas/.jdks/ms-25.0.4.1 ./java_matsim/mvnw \
#    -f ./java_matsim/pom.xml -e -DskipTests \
#    compile exec:java \
#    -Dexec.mainClass=org.matsim.ma_andreas.RunSingleBraessNoSpillbackIteration \
#    "-Dexec.args=$exec_args"
  if ! JAVA_HOME=/home/andreas/.jdks/ms-25.0.4.1 ./java_matsim/mvnw -f ./java_matsim/pom.xml -e \
    -DskipTests compile exec:java -Dexec.mainClass=org.matsim.ma_andreas.RunSingleBraessNoSpillbackIteration \
    "-Dexec.args=$exec_args"
  then
    echo "failed to run java run with parameters "
    record_failure java_experiment "$replanning_variant" "$beta" "$java_seed_index" "$use_this_seed_for_last_iter"
    return 1
  fi
}

export -f get_original_java_replanning_folder_string

export -f run_java_single_iter_case

