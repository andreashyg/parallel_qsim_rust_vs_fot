import sys

import pandas as pd
import matplotlib.pyplot as plt

from setup import FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time, ExperimentSet

# This is the pattern for the paths to the plots created by this script.
plot_type_specific_tt_path_pattern = "{common_plots_pattern}/per_seed/tt_per_path_over_deptime/tt_per_path_over_deptime{file_name_end}.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/per_seed/sd_per_path_over_time/sd_per_path_over_time{file_name_end}.pdf"

if __name__ == '__main__':
    _, beta, replanning_variant, read_random, use_random, experiment_set_name, base_output_dir, input_tt_csv_path_pattern, input_sd_csv_path_pattern = sys.argv
    # FIXME maybe add proper parsing, so things have the right types

    if use_random.lower() == "none":
        use_random = None
    else:
        use_random = int(use_random)

    beta = int(beta)
    read_random = int(read_random)

    # experiment_set_type = ExperimentSet.get(experiment_set_name)
    # experiment_set = experiment_set_type(base_output_dir, replanning_variant, output_tt_plot_path_pattern,
    #                                      output_sd_plot_path_pattern,
    #                                      input_tt_csv_path_pattern, input_sd_csv_path_pattern)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)

    experiment_set = ExperimentSet(base_output_dir,
                                   replanning_variant,
                                   experiment_set_name,
                                   output_tt_plot_path_pattern,
                                   output_sd_plot_path_pattern,
                                   input_tt_csv_path_pattern,
                                   input_sd_csv_path_pattern)

    # # if read_original_java.lower() == "true":
    # if seeds_to_avg_over.lower() == "recreating_java_results":
    #     read_original_java = True
    # else:
    #     read_original_java = False
    #
    # if read_original_java:
    #     file_name_end = f"_beta{beta}_read_from_random_{read_random}_reformatted_original_java_data.csv"
    #
    # else:
    #     # common for both tt and sd .csv file
    #     file_name_end = f"_beta{beta}_read_from_random_{read_random}_use_random_seed_{use_random}.csv"

    # FIXME this (pattern) should be defined in the config as well, since it is used e.g. here and in the rust extraction script
    tt_path = experiment_set.get_path_to_tt_csv_to_read(beta, read_random, use_random)
    sd_path = experiment_set.get_path_to_sd_csv_to_read(beta, read_random, use_random)

    # tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/{seeds_to_avg_over}/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    # sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/{seeds_to_avg_over}/analysis/extracted_data/summed_deps_per_time" + file_name_end

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_tt, mode="tt")
    # read tt data
    tt_df = pd.read_csv(tt_path)
    plot_extracted_tt_over_time(ax_tt, tt_df)

    experiment_set.create_plot_dirs(beta, read_random, use_random)

    # try:
    #     os.makedirs(output_dir + "/tt_per_path_over_deptime")
    # except FileExistsError:
    #     pass

    # if read_original_java:
    #     fig_tt.savefig(
    #         output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_random}_original_java_data.pdf")
    # else:
    fig_tt.savefig(
        experiment_set.get_tt_plot_path(beta, read_random, use_random))
    # output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_random}_use_random_seed{use_random}.pdf")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_sd, mode="sd")
    # read sd data
    sd_df = pd.read_csv(sd_path)

    plot_extracted_sd_over_time(ax_sd, sd_df)

    # try:
    #     os.makedirs(output_dir + "/sd_per_path_over_time")
    # except FileExistsError:
    #     pass

    # if read_original_java:
    #     fig_sd.savefig(
    #         output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_random}_original_java_data.pdf")
    # else:
    fig_sd.savefig(
        experiment_set.get_sd_plot_path(beta, read_random, use_random))
    # output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_random}_use_random_seed{use_random}.pdf")
