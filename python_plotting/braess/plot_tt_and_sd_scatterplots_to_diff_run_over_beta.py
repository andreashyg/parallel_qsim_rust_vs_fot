import sys

from matplotlib import pyplot as plt

from utils import plot_scatter_over_beta, compute_deviation_to_reference_series_but_avg_first, ExperimentSet
from setup import FIG_SIZE

if __name__ == '__main__':
    # _, replanning_variant, fixed_file_dir, dir_to_avg, output_dir = sys.argv
    (_, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir, output_tt_plot_path_pattern,
     output_sd_plot_path_pattern, main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern, betas_to_use_str) = sys.argv[0:13]

    seeds_to_use = [int(s) for s in sys.argv[13:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    if use_random.lower() == "use_all":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random.lower() != "use_all":
            raise ValueError(
                "At least one of read_random or use_random must be 'use_all' to create boxplots using all seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)

    if read_random.lower() == "use_all":
        read_random = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)

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

    # if fixed_file_dir != "recreating_java_results":
    #     raise ValueError(f"fixed_file_dir must be 'recreating_java_results', but got {fixed_file_dir}")
    # fixed_file_end = ("_beta{beta}_"
    #                   + f"read_from_random_{JAVA_SEED_INDEX_WHEN_FIXED}"
    #                   + "_reformatted_original_java_data.csv"
    #                   )
    # if dir_to_avg == "recreating_java_results":
    #     raise ValueError(f"dir_to_avg must not be 'recreating_java_results', but got {dir_to_avg}")
    # file_pattern_to_avg = ("_beta{beta}_" + f"read_from_random_{JAVA_SEED_INDEX_WHEN_FIXED}_use_random_seed_"
    #                        + "{seed}.csv"  # deliberately not an f-string, used as placeholder later
    #                        )
    #
    # tt_path_fixed = ROOT_DATA_PATH + f"/{replanning_variant}/{fixed_file_dir}/analysis/extracted_data/average_route_tts_per_deptime" + fixed_file_end
    # sd_path_fixed = ROOT_DATA_PATH + f"/{replanning_variant}/{fixed_file_dir}/analysis/extracted_data/summed_deps_per_time" + fixed_file_end
    #
    # tt_path_to_avg = ROOT_DATA_PATH + f"/{replanning_variant}/{dir_to_avg}/analysis/extracted_data/average_route_tts_per_deptime" + file_pattern_to_avg
    # sd_path_to_avg = ROOT_DATA_PATH + f"/{replanning_variant}/{dir_to_avg}/analysis/extracted_data/summed_deps_per_time" + file_pattern_to_avg
    #
    # seeds = RUST_SEEDS_TO_ITERATE_OVER  # use_random seeds for rust are 42..61
    # fixed_seed_string_for_filename = "read_from_random"

    ### TT
    # fig_tt, ax_tt = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    #
    # tt_df = compute_deviation_to_reference_df(tt_path_1, betas=betas_to_use, seeds=seeds_to_use, mode="tt",
    #                                           reference_values_path=tt_path_2)
    #
    # # tt_df = compute_deviation_to_reference_df(tt_path_to_avg, betas=BETAS, seeds=seeds, mode="tt",
    # #                                           reference_values_path=tt_path_fixed)
    #
    # plot_boxplot_over_beta(tt_df, ax_tt, "tt", betas_to_use)

    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=FIG_SIZE)
    tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path_1, betas=betas_to_use,
                                                                        seeds=seeds_to_use,
                                                                        mode="tt",
                                                                        reference_values_path=tt_path_2)
    # tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path_to_avg, betas=BETAS, seeds=seeds,
    #                                                                     mode="tt",
    #                                                                     reference_values_path=tt_path_fixed)

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt", betas_to_use)

    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    # try:
    #     os.makedirs(output_dir + "/tt_avg_deviation_boxplots__rust_minus_java")
    # except FileExistsError:
    #     pass

    # fig_tt.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    # fig_tt.savefig(
    #     output_dir + f"/tt_avg_deviation_boxplots__rust_minus_java/tt_avg_deviation_boxplot.pdf")

    # try:
    #     os.makedirs(output_dir + "/tt_avg_first_deviation_scatterplots__rust_minus_java")
    # except FileExistsError:
    #     pass

    fig_tt_avg.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))
    # fig_tt_avg.savefig(
    #     output_dir + f"/tt_avg_first_deviation_scatterplots__rust_minus_java/tt_avg_first_deviation_scatterplot.pdf"
    # )

    ### SD
    # fig_sd, ax_sd = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    #
    # sd_df = compute_deviation_to_reference_df(sd_path_1, betas=betas_to_use, seeds=seeds_to_use, mode="sd",
    #                                           reference_values_path=sd_path_2)
    # # sd_df = compute_deviation_to_reference_df(sd_path_to_avg, betas=BETAS, seeds=seeds, mode="sd",
    # #                                           reference_values_path=sd_path_fixed)
    #
    # plot_boxplot_over_beta(sd_df, ax_sd, "sd", betas_to_use)

    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=FIG_SIZE)
    sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path_1, betas=betas_to_use,
                                                                        seeds=seeds_to_use,
                                                                        mode="sd",
                                                                        reference_values_path=sd_path_2)

    # sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path_to_avg, betas=BETAS, seeds=seeds,
    #                                                                     mode="sd",
    #                                                                     reference_values_path=sd_path_fixed)

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd", betas_to_use)

    # try:
    #     os.makedirs(output_dir + "/sd_avg_deviation_boxplots__rust_minus_java")
    # except FileExistsError:
    #     pass

    # fig_sd.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))
    #
    # fig_sd.savefig(
    #     output_dir + f"/sd_avg_deviation_boxplots__rust_minus_java/sd_avg_deviation_boxplot.pdf")
    #
    # try:
    #     os.makedirs(output_dir + "/sd_avg_first_deviation_scatterplots__rust_minus_java")
    # except FileExistsError:
    #     pass
    #
    fig_sd_avg.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))

    # fig_sd_avg.savefig(
    #     output_dir + f"/sd_avg_first_deviation_scatterplots__rust_minus_java/sd_avg_first_deviation_scatterplot.pdf"
    # )
