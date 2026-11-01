import sys

from matplotlib import pyplot as plt

from setup import PlotParameters, get_file_name_end_and_common_plots_patterns_from_global_config
from utils import plot_scatter_over_beta, compute_deviation_to_reference_series_but_avg_first, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviations{which_deviation}/ttAvgFirstDeviation{which_deviation}Scatterplots/ttAvgFirstDeviation{which_deviation}Scatterplot.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviations{which_deviation}/sdAvgFirstDeviation{which_deviation}Scatterplots/sdAvgFirstDeviation{which_deviation}Scatterplot.pdf"

if __name__ == '__main__':
    (_, experiment_set_name, read_random, use_random, base_output_dir,
     which_deviation, main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern, betas_to_use_str,
     global_config_path,
     sd_ybotlim, sd_ytoplim, tt_ybotlim, tt_ytoplim) = sys.argv[0:16]

    seeds_to_use = [int(s) for s in sys.argv[16:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    sd_ybotlim = float(sd_ybotlim)
    sd_ytoplim = float(sd_ytoplim)
    tt_ybotlim = float(tt_ybotlim)
    tt_ytoplim = float(tt_ytoplim)

    (FILE_NAME_END_PATTERN_WITH_BETA_RR_UR,
     FILE_NAME_END_PATTERN_WITH_BETA_RR,
     FILE_NAME_END_PATTERN_WITH_BETA,
     COMMON_PLOTS_PATTERN) = get_file_name_end_and_common_plots_patterns_from_global_config(global_config_path)

    if use_random == "AvgOverAll":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random != "AvgOverAll":
            raise ValueError(
                "At least one of read_random or use_random must be 'AvgOverAll' to create scatterplots averaging over all seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)

    if read_random == "AvgOverAll":
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

    experiment_set = ExperimentSet(global_config_path, base_output_dir, experiment_set_name,
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

    # TT
    tt_plot_parameters = PlotParameters.load_from_global_config(global_config_path, ybotlim=tt_ybotlim,
                                                                ytoplim=tt_ytoplim)  # 0.0, 0.19
    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=tt_plot_parameters.SCATTERPLOT_FIG_SIZE)
    tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_plot_parameters, tt_path_1,
                                                                        betas=betas_to_use,
                                                                        seeds=seeds_to_use,
                                                                        mode="tt",
                                                                        reference_values_path=tt_path_2)

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt", betas_to_use, plot_params=tt_plot_parameters)

    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    fig_tt_avg.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    # SD
    sd_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, ybotlim=sd_ybotlim,
                                                                ytoplim=sd_ytoplim)  # 0.0, 0.055
    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=sd_plot_parameters.SCATTERPLOT_FIG_SIZE)
    sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_plot_parameters, sd_path_1,
                                                                        betas=betas_to_use,
                                                                        seeds=seeds_to_use,
                                                                        mode="sd",
                                                                        reference_values_path=sd_path_2)

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd", betas_to_use, plot_params=sd_plot_parameters)

    fig_sd_avg.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))
