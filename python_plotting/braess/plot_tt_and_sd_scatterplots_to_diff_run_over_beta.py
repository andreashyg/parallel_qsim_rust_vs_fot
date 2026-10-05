import sys
from matplotlib import pyplot as plt

from setup import FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_scatter_over_beta, compute_deviation_to_reference_series_but_avg_first, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviations_{which_deviation}/tt_avg_first_deviation_{which_deviation}_scatterplots/tt_avg_first_deviation_{which_deviation}_scatterplot.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviations_{which_deviation}/sd_avg_first_deviation_{which_deviation}_scatterplots/sd_avg_first_deviation_{which_deviation}_scatterplot.pdf"

if __name__ == '__main__':
    (_, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir,
     which_deviation, main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern, betas_to_use_str) = sys.argv[0:12]

    seeds_to_use = [int(s) for s in sys.argv[12:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    if use_random.lower() == "avg_over_all":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random.lower() != "avg_over_all":
            raise ValueError(
                "At least one of read_random or use_random must be 'avg_over_all' to create scatterplots averaging over all seeds.")
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
    # also replace the extra which_deviation placeholder
    output_tt_plot_path_pattern = output_tt_plot_path_pattern.replace("{which_deviation}", which_deviation)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra which_deviation placeholder
    output_sd_plot_path_pattern = output_sd_plot_path_pattern.replace("{which_deviation}", which_deviation)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern, output_sd_plot_path_pattern,
                                   main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
                                   secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern)

    tt_path_1 = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=False)
    sd_path_1 = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=False)
    tt_path_2 = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=True)
    sd_path_2 = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=True)

    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=FIG_SIZE)
    tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path_1, betas=betas_to_use,
                                                                        seeds=seeds_to_use,
                                                                        mode="tt",
                                                                        reference_values_path=tt_path_2)

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt", betas_to_use)

    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    fig_tt_avg.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=FIG_SIZE)
    sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path_1, betas=betas_to_use,
                                                                        seeds=seeds_to_use,
                                                                        mode="sd",
                                                                        reference_values_path=sd_path_2)

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd", betas_to_use)

    fig_sd_avg.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))
