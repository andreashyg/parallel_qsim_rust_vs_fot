import matplotlib.pyplot as plt
import pandas as pd
import sys

from setup import FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time, ExperimentSet

# This is the pattern for the paths to the plots created by this script.
plot_type_specific_tt_path_pattern = "{common_plots_pattern}/perSeed/ttPerPathOverDeptime/ttPerPathOverDeptime{file_name_end}.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/perSeed/sdPerPathOverTime/sdPerPathOverTime{file_name_end}.pdf"

if __name__ == '__main__':
    _, beta, replanning_variant, read_random, use_random, experiment_set_name, base_output_dir, input_tt_csv_path_pattern, input_sd_csv_path_pattern = sys.argv

    if use_random.lower() == "none":
        use_random = None
    else:
        use_random = int(use_random)

    beta = int(beta)
    read_random = int(read_random)

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

    tt_path = experiment_set.get_path_to_tt_csv_to_read(beta, read_random, use_random)
    sd_path = experiment_set.get_path_to_sd_csv_to_read(beta, read_random, use_random)

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_tt, mode="tt")
    # read tt data
    tt_df = pd.read_csv(tt_path)
    plot_extracted_tt_over_time(ax_tt, tt_df)

    experiment_set.create_plot_dirs(beta, read_random, use_random)

    fig_tt.savefig(
        experiment_set.get_tt_plot_path(beta, read_random, use_random))

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_sd, mode="sd")
    # read sd data
    sd_df = pd.read_csv(sd_path)

    plot_extracted_sd_over_time(ax_sd, sd_df)

    fig_sd.savefig(
        experiment_set.get_sd_plot_path(beta, read_random, use_random))
