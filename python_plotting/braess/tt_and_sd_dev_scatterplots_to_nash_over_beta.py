import sys
from matplotlib import pyplot as plt

from setup import COMMON_PLOTS_PATTERN, SCATTERPLOT_FIG_SIZE
from utils import plot_scatter_over_beta, compute_deviation_to_reference_series_but_avg_first, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviations_to_nash/tt_avg_deviation_to_nash_scatterplots/tt_avg_deviation_to_nash_scatterplot.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviations_to_nash/sd_avg_deviation_to_nash_scatterplots/sd_avg_deviation_to_nash_scatterplot.pdf"

if __name__ == '__main__':
    (_, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir,
     input_tt_csv_path_pattern, input_sd_csv_path_pattern, betas_to_use_str) = sys.argv[
        0:9]
    seeds_to_use = [int(s) for s in sys.argv[9:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    if use_random.lower() == "avg_over_all":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random.lower() != "avg_over_all":
            raise ValueError("At least one of read_random or use_random must be 'avg_over_all' to average over seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)

    if read_random.lower() == "avg_over_all":
        read_random = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern,
                                   output_sd_plot_path_pattern, input_tt_csv_path_pattern, input_sd_csv_path_pattern)

    tt_path = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                        read_random, use_random)
    sd_path = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                        read_random, use_random)

    # Note: if the plot paths contain placeholders for {beta}, it will be replaced by {beta} again, so no change.
    # While this might be unexpected, it would not make sense to replace {beta} with a specific value here, since the
    # plot shows values for all betas.
    # The same thing holds for placeholders {use_random_seed} when use_random=avg_over_all, and similarly for read_random=avg_over_all.
    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    ### TT

    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=SCATTERPLOT_FIG_SIZE)
    tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path, betas=betas_to_use, seeds=seeds_to_use,
                                                                        mode="tt")

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt", betas_to_use)

    fig_tt_avg.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    ### SD

    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=SCATTERPLOT_FIG_SIZE)
    sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path, betas=betas_to_use, seeds=seeds_to_use,
                                                                        mode="sd")

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd", betas_to_use)

    fig_sd_avg.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))
