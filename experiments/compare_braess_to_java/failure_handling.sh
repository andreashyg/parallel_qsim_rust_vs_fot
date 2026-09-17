# Collect failures so the scripts can finish all experiments/extractions and only report problems at the end.
EXPERIMENT_FAILURES=()
EXTRACTION_FAILURES=()
PLOTTING_FAILURES=()
DUMMY_COORDINATE_ADDING_FAILURES=()
REFORMATTING_FAILURES=()
JAVA_EXPERIMENT_FAILURES=()


# function to called to record a failure in either the experiment run or the extraction or plotting run.
# Called with the arguments:
#   kind: either "experiment" or "extraction" or "plotting"
# and
#   replanning_variant, beta, read_from_random, output_dir, use_random_seed
# Will append a string describing the failure to the corresponding array (EXPERIMENT_FAILURES or EXTRACTION_FAILURES
# or PLOTTING_FAILURES).
# If a failure log file is specified, will also append the failure details to it. This is used when running experiments
# in parallel.
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
    java_experiment)
      JAVA_EXPERIMENT_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
      ;;
    *)
      echo "Unknown failure kind: $kind" >&2
      exit 1
      ;;
  esac

  # If a failure log file is specified, append the failure details to it. This is used when running experiments in parallel
  if [ -n "${FAILURE_LOG_FILE:-}" ]; then
    printf '%s\t%s\t%s\t%s\t%s\n' \
      "$kind" "$replanning_variant" "$beta" "$read_from_random" "$use_random_seed" >> "$FAILURE_LOG_FILE"
  fi
}

import_failure_log() {
  local failure_log_file="$1"
  local kind replanning_variant beta read_from_random use_random_seed

  [ -s "$failure_log_file" ] || return 0

  while IFS=$'\t' read -r kind replanning_variant beta read_from_random use_random_seed; do
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
      java_experiment)
        JAVA_EXPERIMENT_FAILURES+=("${replanning_variant} beta=${beta} read_from_random=${read_from_random} use_random_seed=${use_random_seed}")
        ;;
      *)
        echo "Unknown failure kind in failure log: $kind" >&2
        ;;
    esac
  done < "$failure_log_file"
}

# function to print a summary of all failures recorded during the batch run. Will print the number of failures and the
# details of each failure.
print_failure_summary() {
  if [ "${#EXPERIMENT_FAILURES[@]}" -eq 0 ] && [ "${#EXTRACTION_FAILURES[@]}" -eq 0 ] && [ "${#PLOTTING_FAILURES[@]}" -eq 0 ] && [ "${#DUMMY_COORDINATE_ADDING_FAILURES[@]}" -eq 0 ] && [ "${#REFORMATTING_FAILURES[@]}" -eq 0 ] && [ "${#JAVA_EXPERIMENT_FAILURES[@]}" -eq 0 ]; then
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

  if [ "${#JAVA_EXPERIMENT_FAILURES[@]}" -gt 0 ]; then
    echo "  Java experiment failures (${#JAVA_EXPERIMENT_FAILURES[@]}):"
    for failure in "${JAVA_EXPERIMENT_FAILURES[@]}"; do
      echo "    - $failure"
    done
  fi
}

__BATCH_COMMON_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export -f record_failure
export -f import_failure_log
export -f print_failure_summary