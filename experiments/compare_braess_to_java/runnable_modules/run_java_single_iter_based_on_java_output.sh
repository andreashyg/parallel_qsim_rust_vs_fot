#!/bin/bash

set -uo pipefail

source "${SCRIPT_DIR}/../failure_handling.sh"

# function to read config file entries into a variable.
# Read the global config file.
# The first argument is the name of the variable to read into, the second argument is the key in the config file.
read_config_file_entry() {
  local -n var_to_change="$1"
  local key="$2"
  local global_config_path="$3"

  echo will be looking for key $key in global config file $global_config_path
  mapfile -t var_to_change < <(yq -r ".global_parameters.${key} | if type == \"array\" then .[] else . end" "$global_config_path")
}


#get_original_java_replanning_folder_string() {
#  local replanning_variant="$1"
#
#  if [ "${replanning_variant}" = "sel-exp1-switch-at50" ]; then
#    echo "2026-05-8-12-16-8_500it_reRouteProba0.1until0.5it_selExpBeta1proba0.9_msaFrom0.5it"
#  elif [ "${replanning_variant}" = "sel-exp1-switch-at80" ]; then
#    echo "2026-05-10-10-2-21_500it_reRouteProba0.1until0.8it_selExpBeta1proba0.9_msaFrom0.8it"
#  elif [ "${replanning_variant}" = "sel-exp10-switch-at80" ]; then
#    echo "2026-05-12-8-42-24_500it_reRouteProba0.1until0.8it_selExpBeta10proba0.9_msaFrom0.8it"
#  else
#    echo "unknown replanning_variant value: $replanning_variant" >&2
#    return 1
#  fi
#}

# function to run a single experiment case.
run_java_single_iter_case() {
  local beta="$1"
  local read_from_random_seed_index="$2"
  local use_this_seed_for_last_iter="$3"
  local global_config_path="$4"
  local experiment_set_name="$5"
  local base_output_dir="$6"
  local delete_output_dir_if_existing="$7"
  local pop_to_use_as_input="$8"
  local config_file_pattern="$9"

  local delete_output_dir_arg=()
  if [ "$delete_output_dir_if_existing" = true ]; then
    delete_output_dir_arg=(--delete-output-dir-if-existing)
  fi

  # read from global config here and pass to java, to avoid having to make java read yaml files
  read_config_file_entry experiment_output_dir_pattern common_output_from_runs_pattern "$global_config_path"
  read_config_file_entry base_dir_to_read_from original_dir_to_read_from "$global_config_path"

#  local original_java_replanning_variant_string
#  original_java_replanning_variant_string=$(get_original_java_replanning_folder_string "$replanning_variant")

#  local config_file_pattern
#  read_config_file_entry config_file_pattern original_output_config_pattern global_config_path

  # replace {base_dir_to_read_from}, {original_java_replanning_variant_string}, {beta}, {read_from_random} in config_file_pattern with the corresponding values
  config_file=${config_file_pattern//\{original_dir_to_read_from\}/$base_dir_to_read_from}
#  config_file=${config_file//\{replanning_variant\}/$original_java_replanning_variant_string}
  config_file=${config_file//\{beta\}/$beta}
  config_file=${config_file//\{read_from_random\}/$read_from_random_seed_index}

#  local pop_to_use_as_input
#  read_config_file_entry pop_to_use_as_input pop_to_use_as_input_pattern global_config_path

  # note: we are not just replacing {base_output_dir} with $base_output_dir, but with ../../../../../../$base_output_dir,
  # because java matsim needs paths relative to the config that it's reading.
  pop_to_use_as_input=${pop_to_use_as_input//\{base_output_dir\}/../../../../../../../$base_output_dir}
  # same thing for original dir to read from (since the plans could be read from either the base output dir or the original dir to read from)
  pop_to_use_as_input=${pop_to_use_as_input//\{original_dir_to_read_from\}/../../../../../$base_dir_to_read_from}

#  pop_to_use_as_input=${pop_to_use_as_input//\{replanning_variant\}/$replanning_variant}
  pop_to_use_as_input=${pop_to_use_as_input//\{experiment_set_name\}/$experiment_set_name}
  pop_to_use_as_input=${pop_to_use_as_input//\{beta\}/$beta}
  pop_to_use_as_input=${pop_to_use_as_input//\{read_from_random\}/$read_from_random_seed_index}

  local experiment_output_dir
  # replace {base_output_dir}, {replanning_variant}, {experiment_set_name}, {beta}, {read_from_random_seed_index}, {use_this_seed_for_last_iter} in experiment_output_dir_pattern with the corresponding values
  experiment_output_dir=${experiment_output_dir_pattern//\{base_output_dir\}/$base_output_dir}
#  experiment_output_dir=${experiment_output_dir//\{replanning_variant\}/$replanning_variant}
  experiment_output_dir=${experiment_output_dir//\{experiment_set_name\}/$experiment_set_name}
  experiment_output_dir=${experiment_output_dir//\{beta\}/$beta}
  experiment_output_dir=${experiment_output_dir//\{read_from_random\}/$read_from_random_seed_index}
  experiment_output_dir=${experiment_output_dir//\{use_random_seed\}/$use_this_seed_for_last_iter}


  java_args=(
    run
    --config "$config_file"
    --correctedPopFile "$pop_to_use_as_input"
    --beta "$beta"
    --useRandom "$use_this_seed_for_last_iter"
    --outputDir "$experiment_output_dir"
    "${delete_output_dir_arg[@]}"
  )

  # Array -> sauber gequoteter String für -Dexec.args
  printf -v exec_args '%q ' "${java_args[@]}"
  exec_args=${exec_args% }

  if ! JAVA_HOME=/home/andreas/.jdks/ms-25.0.4.1 ./java_matsim/mvnw -f ./java_matsim/pom.xml -e \
    -DskipTests exec:java -Dexec.mainClass=org.matsim.ma_andreas.RunSingleBraessNoSpillbackIteration \
    "-Dexec.args=$exec_args"
  then
    echo "failed to run java run with parameters "
    record_failure java_experiment "deprecated" "$beta" "$read_from_random_seed_index" "$use_this_seed_for_last_iter"
    return 1
  fi
}

#export -f get_original_java_replanning_folder_string
export -f read_config_file_entry

export -f run_java_single_iter_case

