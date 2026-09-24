import sys
from matplotlib import pyplot as plt

from setup import BOXPLOT_FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_boxplot_over_beta, compute_deviation_to_reference_df, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviations_to_nash/tt_avg_deviation_to_nash_boxplots/tt_avg_deviation_to_nash_boxplot.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviations_to_nash/sd_avg_deviation_to_nash_boxplots/sd_avg_deviation_to_nash_boxplot.pdf"

if __name__ == '__main__':
    # _, replanning_variant, seeds_to_avg_over, output_dir, read_original_java = sys.argv
    (_, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir,
     input_tt_csv_path_pattern, input_sd_csv_path_pattern, betas_to_use_str) = sys.argv[0:9]
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

    # if read_original_java.lower() == "true":
    #     read_original_java = True
    # else:
    #     read_original_java = False

    tt_path = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                        read_random, use_random)
    sd_path = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                        read_random, use_random)

    # if seeds_to_avg_over == "java":
    #     if read_original_java:
    #         # common for both tt and sd .csv file
    #         file_name_end = (
    #             # the plots will average over beta and read_from_random
    #                 "_beta{beta}_read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
    #                 + "_reformatted_original_java_data.csv"  # we only use one random seed in rust
    #         )
    #         tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/recreating_java_results/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    #         sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/recreating_java_results/analysis/extracted_data/summed_deps_per_time" + file_name_end
    #
    #     else:
    #         # common for both tt and sd .csv file
    #         file_name_end = (
    #             # the plots will average over beta and read_from_random
    #                 "_beta{beta}_read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
    #                 + f"_use_random_seed_{RUST_SEED_WHEN_FIXED}.csv"  # we only use one random seed in rust
    #         )
    #         tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    #         sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end
    #
    #     seeds = JAVA_SEED_INDICES_TO_ITERATE_OVER  # read_from_random seeds are 1..20
    # elif seeds_to_avg_over == "rust":
    #     if read_original_java:
    #         raise ValueError(
    #             "Parameter combination read_original_java=True and seeds_to_avg_over=rust is not supported")
    #
    #     # the plots will average over beta and use_random_seed (deliberately only one of the strings is f-string)
    #     file_name_end = "_beta{beta}" + f"_read_from_random_{JAVA_SEED_INDEX_WHEN_FIXED}" + "_use_random_seed_{seed}.csv"
    #
    #     tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    #     sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end
    #
    #     seeds = RUST_SEEDS_TO_ITERATE_OVER  # use_random_seeds are 42..61
    # else:
    #     raise ValueError("Invalid value for 'which_seeds', must be 'java' or 'rust'")

    # Note: if the plot paths contain placeholders for {beta}, it will be replaced by {beta} again, so no change.
    # While this might be unexpected, it would not make sense to replace {beta} with a specific value here, since the
    # plot shows values for all betas.
    # The same thing holds for placeholders {use_random_seed} when use_random=avg_over_all, and similarly for read_random=avg_over_all.
    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    tt_df = compute_deviation_to_reference_df(tt_path, betas=betas_to_use, seeds=seeds_to_use, mode="tt")

    plot_boxplot_over_beta(tt_df, ax_tt, "tt", betas_to_use)

    fig_tt.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    # fig_tt_avg, ax_tt_avg = plt.subplots(figsize=FIG_SIZE)
    # tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path, betas=BETAS, seeds=seeds_to_use,
    #                                                                     mode="tt")

    # plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt")

    # try:
    #     os.makedirs(output_dir + "/tt_avg_deviation_boxplots")
    # except FileExistsError:
    #     pass
    #
    # fig_tt.savefig(
    #     output_dir + f"/tt_avg_deviation_boxplots/tt_avg_deviation_boxplot.pdf")

    # fig_tt_avg.savefig(experiment_set.get_tt_plot_dir_path("{beta}", read_random, use_random))

    # try:
    #     os.makedirs(output_dir + "/tt_avg_first_deviation_scatterplots")
    # except FileExistsError:
    #     pass

    # # TODO: the experiment set so far only supports one plot, even though here, we have two cases.
    # #  should it maybe be split into two files/modules?
    # fig_tt_avg.savefig(
    #     output_dir + f"/tt_avg_first_deviation_scatterplots/tt_avg_first_deviation_scatterplot.pdf"
    # )

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    sd_df = compute_deviation_to_reference_df(sd_path, betas=betas_to_use, seeds=seeds_to_use, mode="sd")

    plot_boxplot_over_beta(sd_df, ax_sd, "sd", betas_to_use)

    fig_sd.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))

    # fig_sd_avg, ax_sd_avg = plt.subplots(figsize=FIG_SIZE)
    # sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path, betas=BETAS, seeds=seeds, mode="sd")
    #
    # plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd")

    # try:
    #     os.makedirs(output_dir + "/sd_avg_deviation_boxplots")
    # except FileExistsError:
    #     pass
    #
    # fig_sd.savefig(
    #     output_dir + f"/sd_avg_deviation_boxplots/sd_avg_deviation_boxplot.pdf")
    #
    # try:
    #     os.makedirs(output_dir + "/sd_avg_first_deviation_scatterplots")
    # except FileExistsError:
    #     pass
    #
    # fig_sd_avg.savefig(
    #     output_dir + f"/sd_avg_first_deviation_scatterplots/sd_avg_first_deviation_scatterplot.pdf"
    # )
