import sys

import matplotlib.pyplot as plt

import pandas as pd
from setup import FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_extracted_sd_over_time, plot_extracted_tt_over_time, get_elementwise_avg_df, \
    get_elementwise_difference_df, plot_value_count_table, plot_textbox, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/avg_over_seeds/tt_per_path_over_deptime_minus_{minus_what}/tt_per_path_over_deptime_minus_{minus_what}{file_name_end}.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/avg_over_seeds/sd_per_path_over_time_minus_{minus_what}/sd_per_path_over_time_minus_{minus_what}{file_name_end}.pdf"

if __name__ == '__main__':

    # _, beta, replanning_variant, fixed_seed, fixed_file_dir, dir_to_avg, output_dir = sys.argv
    (_, beta, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir,
     minus_what, main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern) = sys.argv[0:12]

    seeds_to_use = [int(s) for s in sys.argv[12:]]

    beta = int(beta)
    if use_random.lower() == "avg_over_all":
        use_random_formatting_term_with_optional_placeholder = "{seed}"  # placeholder to be formatted later
        # use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random.lower() != "avg_over_all":
            raise ValueError("At least one of read_random or use_random must be 'avg_over_all' to average over seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)
        use_random_formatting_term_with_optional_placeholder = use_random  # no placeholder (fixed value)

    if read_random.lower() == "avg_over_all":
        # read_random = "{seed}"  # placeholder to be formatted later
        read_random_formatting_term_with_optional_placeholder = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)
        read_random_formatting_term_with_optional_placeholder = read_random  # no placeholder (fixed value)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra minus_what placeholder
    output_tt_plot_path_pattern = output_tt_plot_path_pattern.replace("{minus_what}", minus_what)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra minus_what placeholder
    output_sd_plot_path_pattern = output_sd_plot_path_pattern.replace("{minus_what}", minus_what)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern, output_sd_plot_path_pattern,
                                   main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
                                   secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern)

    tt_path_1 = experiment_set.get_path_to_tt_csv_to_read(beta, read_random_formatting_term_with_optional_placeholder,
                                                          use_random_formatting_term_with_optional_placeholder,
                                                          secondary=False)
    sd_path_1 = experiment_set.get_path_to_sd_csv_to_read(beta, read_random_formatting_term_with_optional_placeholder,
                                                          use_random_formatting_term_with_optional_placeholder,
                                                          secondary=False)
    tt_path_2 = experiment_set.get_path_to_tt_csv_to_read(beta, read_random_formatting_term_with_optional_placeholder,
                                                          use_random_formatting_term_with_optional_placeholder,
                                                          secondary=True)
    sd_path_2 = experiment_set.get_path_to_sd_csv_to_read(beta, read_random_formatting_term_with_optional_placeholder,
                                                          use_random_formatting_term_with_optional_placeholder,
                                                          secondary=True)

    # if fixed_file_dir == "recreating_java_results":
    #     fixed_file_end = (f"_beta{beta}_"
    #                       + f"read_from_random_{fixed_seed}"
    #                       + f"_reformatted_original_java_data.csv"
    #                       )
    # else:
    #     raise ValueError(f"fixed_file_dir must be 'recreating_java_results', but got {fixed_file_dir}")
    #
    # if dir_to_avg == "recreating_java_results":
    #     raise ValueError(f"dir_to_avg must not be 'recreating_java_results', but got {dir_to_avg}")
    #
    # else:
    #     file_pattern_to_avg = (f"_beta{beta}_read_from_random_{fixed_seed}_use_random_seed_"
    #                            + "{seed}.csv"  # deliberately not an f-string, used as placeholder later
    #                            )
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

    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)

    # if the placeholder "{seed}" is in the path, we need to compute the elementwise average over the seeds,
    # otherwise we can just read the csv file (since the seed is apparently fixed)
    if "{seed}" in tt_path_1:
        tt_df_1 = get_elementwise_avg_df(tt_path_1, seeds_to_use=seeds_to_use, mode="tt")
    else:
        tt_df_1 = pd.read_csv(tt_path_1)

    if "{seed}" in tt_path_2:
        tt_df_2 = get_elementwise_avg_df(tt_path_2, seeds_to_use=seeds_to_use, mode="tt")
    else:
        tt_df_2 = pd.read_csv(tt_path_2)

    # get the average of the two dataframes
    # avg_tt_df = get_elementwise_avg_df(tt_path_to_avg, seeds_to_use=list(seeds), mode="tt")
    # get difference of averaged travel times over rust seeds and the fixed original java travel times
    # tt_df = get_elementwise_difference_df(avg_tt_df, pd.read_csv(tt_path_fixed), mode="tt")

    tt_df = get_elementwise_difference_df(tt_df_1, tt_df_2, mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, per_path=False, ybotlim=-3.3, ytoplim=4.7)

    # get series with for each discrete difference, the amount of time steps that have this difference, and the percentage of time steps that have this difference
    # tt_diff_counts = pd.concat([tt_df[f'avg_travel_time_path_{i}'] for i in range(3)]).apply(
    #     lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()
    tt_diff_counts = tt_df["avg_travel_time"].apply(
        lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()

    # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
    plot_value_count_table(ax_tt, tt_diff_counts, val_col_name="Sign")

    tt_total_diff = tt_df["avg_travel_time"].abs().sum() / tt_df["avg_travel_time"].count()

    plot_textbox(ax_tt, f"Average absolute difference: {tt_total_diff:.4f}", x=0.01, y=0.1)

    experiment_set.create_plot_dirs(beta, read_random, use_random)

    # try:
    #     os.makedirs(output_dir + "/tt_per_path_over_deptime__avg_over_rust_minus_java")
    # except FileExistsError:
    #     pass
    fig_tt.savefig(experiment_set.get_tt_plot_path(beta, read_random, use_random))
    # fig_tt.savefig(
    #     output_dir + f"/tt_per_path_over_deptime__avg_over_rust_minus_java/tt_per_path_over_deptime_beta{beta}_{fixed_seed_string_for_filename}_{fixed_seed}.pdf")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)

    if "{seed}" in sd_path_1:
        sd_df_1 = get_elementwise_avg_df(sd_path_1, seeds_to_use=seeds_to_use, mode="sd")
    else:
        sd_df_1 = pd.read_csv(sd_path_1)

    if "{seed}" in sd_path_2:
        sd_df_2 = get_elementwise_avg_df(sd_path_2, seeds_to_use=seeds_to_use, mode="sd")
    else:
        sd_df_2 = pd.read_csv(sd_path_2)

    # avg_sd_df = get_elementwise_avg_df(sd_path_to_avg, seeds_to_use=seeds, mode="sd")
    # get difference of averaged summed departures over rust seeds and the fixed original java summed departures
    # sd_df = get_elementwise_difference_df(avg_sd_df, pd.read_csv(sd_path_fixed), mode="sd")

    sd_df = get_elementwise_difference_df(sd_df_1, sd_df_2, mode="sd")

    plot_extracted_sd_over_time(ax_sd, sd_df, ybotlim=-1, ytoplim=1)

    # get series with for each discrete difference, the amount of time steps that have this difference
    sd_diff_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).value_counts().sort_index()
    # sd_diff_counts = sd_df["sum_departures_total"].apply(
    #     lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()

    # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
    plot_value_count_table(ax_sd, sd_diff_counts)

    # try:
    #     os.makedirs(output_dir + "/sd_per_path_over_time__avg_over_rust_minus_java")
    # except FileExistsError:
    #     pass

    fig_sd.savefig(experiment_set.get_sd_plot_path(beta, read_random, use_random))
    # fig_sd.savefig(
    #     output_dir + f"/sd_per_path_over_time__avg_over_rust_minus_java/sd_per_path_over_time_beta{beta}_{fixed_seed_string_for_filename}_{fixed_seed}.pdf")
