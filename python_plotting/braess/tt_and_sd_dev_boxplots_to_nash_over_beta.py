import sys

from matplotlib import pyplot as plt

from setup import PlotParameters, get_file_name_end_and_common_plots_patterns_from_global_config
from utils import plot_boxplot_over_beta, compute_deviation_to_reference_df, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviationsToNash/ttAvgDeviationToNashBoxplots/ttAvgDeviationToNashBoxplot.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviationsToNash/sdAvgDeviationToNashBoxplots/sdAvgDeviationToNashBoxplot.pdf"

if __name__ == '__main__':
    (_, experiment_set_name, read_random, use_random, base_output_dir,
     input_tt_csv_path_pattern, input_sd_csv_path_pattern, betas_to_use_str, global_config_path,
     sd_ybotlim, sd_ytoplim, tt_ybotlim, tt_ytoplim) = sys.argv[0:13]
    seeds_to_use = [int(s) for s in sys.argv[13:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    sd_ybotlim = float(sd_ybotlim)
    sd_ytoplim = float(sd_ytoplim)
    tt_ybotlim = float(tt_ybotlim)
    tt_ytoplim = float(tt_ytoplim)

    (FILE_NAME_END_PATTERN_WITH_BETA_RR_UR,
     FILE_NAME_END_PATTERN_WITH_BETA_RR,
     FILE_NAME_END_PATTERN_WITH_BETA,
     COMMON_PLOTS_PATTERN) = get_file_name_end_and_common_plots_patterns_from_global_config(global_config_path)

    if use_random == "UseAll":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random != "UseAll":
            raise ValueError(
                "At least one of read_random or use_random must be 'UseAll' to use all seeds in the boxplot.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)

    if read_random == "UseAll":
        read_random = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)

    experiment_set = ExperimentSet(global_config_path, base_output_dir, experiment_set_name,
                                   output_tt_plot_path_pattern,
                                   output_sd_plot_path_pattern, input_tt_csv_path_pattern, input_sd_csv_path_pattern)

    tt_path = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                        read_random, use_random)
    sd_path = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                        read_random, use_random)

    # Note: if the plot paths contain placeholders for {beta}, it will be replaced by {beta} again, so no change.
    # While this might be unexpected, it would not make sense to replace {beta} with a specific value here, since the
    # plot shows values for all betas.
    # The same thing holds for placeholders {use_random_seed} when use_random=UseAll, and similarly for read_random=UseAll.
    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    ### TT
    tt_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, ybotlim=tt_ybotlim,
                                                                ytoplim=tt_ytoplim)  # 0, 8.5
    fig_tt, ax_tt = plt.subplots(figsize=tt_plot_parameters.BOXPLOT_FIG_SIZE)
    tt_df = compute_deviation_to_reference_df(tt_plot_parameters, tt_path, betas=betas_to_use,
                                              seeds=seeds_to_use, mode="tt")

    plot_boxplot_over_beta(tt_df, ax_tt, "tt", betas_to_use, plot_params=tt_plot_parameters)

    fig_tt.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    ### SD
    sd_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, ybotlim=sd_ybotlim,
                                                                ytoplim=sd_ytoplim)  # 0., 3.5
    fig_sd, ax_sd = plt.subplots(figsize=sd_plot_parameters.BOXPLOT_FIG_SIZE)
    sd_df = compute_deviation_to_reference_df(sd_plot_parameters, sd_path, betas=betas_to_use, seeds=seeds_to_use,
                                              mode="sd")

    plot_boxplot_over_beta(sd_df, ax_sd, "sd", betas_to_use, plot_params=sd_plot_parameters)

    fig_sd.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))
