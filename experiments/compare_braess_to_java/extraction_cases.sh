#!/bin/bash

# function to run the travel time & summed departures extraction for a single experiment case. Called with the arguments:
#   replanning_variant, beta, read_from_random, use_random_seed, seeds_to_avg_over, output_dir
# Will read the events written by the simulation in the given output directory and write the average travel times per
# route to a CSV file in the analysis subdirectory of the output root directory.
extract_travel_time_sum_dep_case() {
  local replanning_variant="$1"
  local seeds_to_avg_over="$2"
  local beta="$3"
  local seed_index="$4"
#  local read_from_random="$4"
#  local use_random_seed="$5"
#  local read_original_java="$6"

  # this will set read_from_random to the java seed index based on whether we are averaging over java seeds or rust
  # seeds.
  # In the former case, gets the corresponding java seed index from the array of java seeds to iterate over,
  # in the latter case, gets the fixed java seed index.
  local read_from_random
  read_from_random=$(get_java_seed_from_index "$seeds_to_avg_over" "$seed_index")
  # Same thing for rust seeds
  local use_random_seed
  use_random_seed=$(get_rust_seed_from_index "$seeds_to_avg_over" "$seed_index")

  # output directory of the rust simulations
  local output_dir
  output_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/beta${beta}/read_from_random_${read_from_random}_use_random_seed_${use_random_seed}"

  # file stem for the events file written by the rust simulation
  local input_file_stem="${output_dir}/events/events"

  # directory where the measured data (travel times and summed departures) will be extracted to.
  local extracted_data_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/extracted_data"

  # if that directory already exists for some parameters, skip if the corresponding flag is set
  if [ -d extracted_data_dir ] && [ "${skip_existing_output_dir}" = "true" ]; then
    echo "Skipping travel time/summed departures extraction because the output directory already exists: $extracted_data_dir"
    return 0
  fi

  # Paths to the tt and sd files to be written.
  local tt_csv_path="${extracted_data_dir}/average_route_tts_per_deptime_beta${beta}_read_from_random_${read_from_random}_use_random_seed_${use_random_seed}.csv"
  local sd_csv_path="${extracted_data_dir}/summed_deps_per_time_beta${beta}_read_from_random_${read_from_random}_use_random_seed_${use_random_seed}.csv"

  echo "Extracting average travel times and summed departures for parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random=$read_from_random, use_random_seed=$use_random_seed, seeds_to_avg_over=$seeds_to_avg_over"
  echo "writing into $tt_csv_path and $sd_csv_path"

  # Extract travel times and summed departures from the events written by the rust simulation.
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

try_extracting_measurements_from_java_case() {
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

  # directory where the measured data (travel times and summed departures) will be extracted to.
  # This is the same for the original java runs and the rust runs, since we want to compare them
  # in the same (kind of) plots.
  local extracted_data_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/varying_${seeds_to_avg_over}_seeds/analysis/extracted_data"

  local tt_csv_path_newextraction="${extracted_data_dir}/average_route_tts_per_deptime_beta${beta}_read_from_random_${read_from_random}_original_java_data_newextraction.csv"
  local sd_csv_path_newextraction="${extracted_data_dir}/summed_deps_per_time_beta${beta}_read_from_random_${read_from_random}_original_java_data_newextraction.csv"

  # folder name of the replanning variant in the original java data
  local replanning_str
  replanning_str=$(get_original_java_replanning_folder_string "$replanning_variant")

  # events file from the original java run
  local original_input_file="${SIM_OUTPUT_BASE_DIR}/../../../../braess/refinement/no_spillback_scenario/${replanning_str}/beta${beta}/random${read_from_random}/beta${beta}random${read_from_random}.output_events.xml.gz"
  # output file stem for the events file with added dummy coordinates, which will be created by the
  # simple_dummy_coordinate_adder binary in the next step
  # This is needed because the original java events file does not contain coordinates, which are needed for the
  # travel time and summed departures extraction (Rust doesn't accept event files without coordinates).
  local new_output_file_stem="${SIM_OUTPUT_BASE_DIR}/../../../../Abschlussarbeiten/2026/andreas-hygrell-rust-vs-fot/compare_braess_to_java/${replanning_variant}/java_event_files_with_added_dummy_coords/beta${beta}/random${read_from_random}/beta${beta}random${read_from_random}.output_events_with_dummy_coords"

  # Often, this will fail, since output events of the Java runs are often not available.
  if ! cargo run --release --bin simple_dummy_coordinate_adder -- \
    --input-file "${original_input_file}" \
    --output-file "${new_output_file_stem}.xml.gz"
  then
    echo "Failed to add dummy coordinates to the original Java output events." >&2
    record_failure dummy_coordinate_adding "$replanning_variant" "$beta" "$read_from_random" "reading_original_java"
    return 1
  fi

  # Extract travel times and summed departures from the events (w/ added coords) from the original java runs
  if ! cargo run --release --bin event_data_extractor -- \
    --input-file-stem "$new_output_file_stem" \
    --input-file-format "xml.gz" \
    --tt-csv-path "$tt_csv_path_newextraction" \
    --sd-csv-path "$sd_csv_path_newextraction" \
    --num-parts 0 \
    --link-to-path-map-name "braess" \
    --beta "$beta"
  then
    echo "Travel-time/summed departures extraction failed, continuing with next case." >&2
    record_failure extraction "$replanning_variant" "$beta" "$read_from_random" "reading_original_java"
    return 1
  fi
}

reformat_original_java_extracted_measurements_case() {
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

  # directory where the reformatted measured data will be stored
  local extracted_data_dir="${SIM_OUTPUT_BASE_DIR}/${replanning_variant}/recreating_java_results/analysis/extracted_data"

  # Paths to the tt and sd files to be written.
  local tt_csv_path="${extracted_data_dir}/average_route_tts_per_deptime_beta${beta}_read_from_random_${read_from_random}_reformatted_original_java_data.csv"
  local sd_csv_path="${extracted_data_dir}/summed_deps_per_time_beta${beta}_read_from_random_${read_from_random}_reformatted_original_java_data.csv"

  echo "Copying and reformatting extracted original java average travel times and summed departures for parameters: replanning_variant=$replanning_variant, beta=$beta, read_from_random=$read_from_random"

  # folder name of the replanning variant in the original java data
  local replanning_str
  replanning_str=$(get_original_java_replanning_folder_string "$replanning_variant")

  # Take the TSV files with travel times and summed departures from the Java runs and copy them
  # into CSV files with the same format (and column names) as my own extracted data

  original_java_extracted_tt_file="${SIM_OUTPUT_BASE_DIR}/../../../../braess/refinement/no_spillback_scenario/${replanning_str}/analysis/average_traveltimes/avgRouteTTsPerDeparture_${beta}_${read_from_random}_500.txt"
  original_java_extracted_sd_file="${SIM_OUTPUT_BASE_DIR}/../../../../braess/refinement/no_spillback_scenario/${replanning_str}/analysis/summed_departures/summedDeparturesPerRoute_${beta}_${read_from_random}_500.txt"

  if ! cargo run --release --bin simple_csv_column_renamer -- \
    --input-file "${original_java_extracted_tt_file}" \
    --output-file "${tt_csv_path}" --column-name departure_time \
    --column-name avg_travel_time_path_0 --column-name avg_travel_time_path_1 \
    --column-name avg_travel_time_path_2 --column-name avg_travel_time || ! cargo run --release --bin \
    simple_csv_column_renamer -- \
    --input-file "${original_java_extracted_sd_file}" \
    --output-file "${sd_csv_path}" \
    --column-name time --column-name sum_departures_path_0 \
    --column-name sum_departures_path_1 --column-name sum_departures_path_2 \
    --column-name sum_departures_total
  then
    echo "Failed to copy and reformat the original java extracted data." >&2
    record_failure reformatting "$replanning_variant" "$beta" "$read_from_random" "reading_original_java"
    return 1
  fi

}
