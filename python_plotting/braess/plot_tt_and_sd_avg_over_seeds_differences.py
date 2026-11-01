import sys

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

from setup import PlotParameters, get_file_name_end_and_common_plots_patterns_from_global_config
from utils import plot_extracted_sd_over_time, plot_extracted_tt_over_time, get_elementwise_avg_df, \
    get_elementwise_difference_df, plot_value_count_table, plot_textbox, ExperimentSet, save_value_count_table_as_csv

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/avgOverSeeds/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/avgOverSeeds/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}.pdf"

tt_diff_sign_count_csv_path = "{common_plots_pattern}/avgOverSeeds/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}SignCount.csv"
sd_diff_sign_count_csv_path = "{common_plots_pattern}/avgOverSeeds/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}SignCount.csv"
tt_avg_diff_csv_path = "{common_plots_pattern}/avgOverSeeds/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}AvgDiff.csv"
tt_avg_abs_diff_csv_path = "{common_plots_pattern}/avgOverSeeds/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}AvgAbsDiff.csv"
sd_avg_diff_csv_path = "{common_plots_pattern}/avgOverSeeds/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}AvgDiff.csv"
sd_avg_abs_diff_csv_path = "{common_plots_pattern}/avgOverSeeds/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}AvgAbsDiff.csv"

if __name__ == '__main__':

    (_, beta, experiment_set_name, read_random, use_random, base_output_dir,
     minus_what, main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern, global_config_path,
     sd_ybotlim, sd_ytoplim, tt_ybotlim, tt_ytoplim) = sys.argv[0:16]

    seeds_to_use = [int(s) for s in sys.argv[16:]]

    sd_ybotlim = float(sd_ybotlim)
    sd_ytoplim = float(sd_ytoplim)
    tt_ybotlim = float(tt_ybotlim)
    tt_ytoplim = float(tt_ytoplim)

    (FILE_NAME_END_PATTERN_WITH_BETA_RR_UR,
     FILE_NAME_END_PATTERN_WITH_BETA_RR,
     FILE_NAME_END_PATTERN_WITH_BETA,
     COMMON_PLOTS_PATTERN) = get_file_name_end_and_common_plots_patterns_from_global_config(global_config_path)

    beta = int(beta)
    if use_random == "AvgOverAll":
        use_random_formatting_term_with_optional_placeholder = "{seed}"  # placeholder to be formatted later
    else:
        if read_random != "AvgOverAll":
            raise ValueError("At least one of read_random or use_random must be 'AvgOverAll' to average over seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)
        use_random_formatting_term_with_optional_placeholder = use_random  # no placeholder (fixed value)

    if read_random == "AvgOverAll":
        read_random_formatting_term_with_optional_placeholder = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)
        read_random_formatting_term_with_optional_placeholder = read_random  # no placeholder (fixed value)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra minus_what placeholder
    output_tt_plot_path_pattern = output_tt_plot_path_pattern.replace("{minus_what}", minus_what)
    output_sd_plot_path_pattern = output_sd_plot_path_pattern.replace("{minus_what}", minus_what)

    # same thing for the csv paths
    tt_diff_sign_count_csv_path = tt_diff_sign_count_csv_path.replace("{common_plots_pattern}",
                                                                      COMMON_PLOTS_PATTERN)
    tt_diff_sign_count_csv_path = tt_diff_sign_count_csv_path.replace("{minus_what}", minus_what)
    sd_diff_sign_count_csv_path = sd_diff_sign_count_csv_path.replace("{common_plots_pattern}",
                                                                      COMMON_PLOTS_PATTERN)
    sd_diff_sign_count_csv_path = sd_diff_sign_count_csv_path.replace("{minus_what}", minus_what)
    tt_avg_abs_diff_csv_path = tt_avg_abs_diff_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    tt_avg_abs_diff_csv_path = tt_avg_abs_diff_csv_path.replace("{minus_what}", minus_what)
    sd_avg_abs_diff_csv_path = sd_avg_abs_diff_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    sd_avg_abs_diff_csv_path = sd_avg_abs_diff_csv_path.replace("{minus_what}", minus_what)

    tt_avg_diff_csv_path = tt_avg_diff_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    tt_avg_diff_csv_path = tt_avg_diff_csv_path.replace("{minus_what}", minus_what)
    sd_avg_diff_csv_path = sd_avg_diff_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    sd_avg_diff_csv_path = sd_avg_diff_csv_path.replace("{minus_what}", minus_what)

    experiment_set = ExperimentSet(global_config_path, base_output_dir, experiment_set_name,
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

    # csv paths

    tt_diff_sign_count_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        tt_diff_sign_count_csv_path, beta, read_random, use_random)
    sd_diff_sign_count_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        sd_diff_sign_count_csv_path, beta, read_random, use_random)
    tt_avg_abs_diff_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        tt_avg_abs_diff_csv_path, beta, read_random, use_random)
    sd_avg_abs_diff_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        sd_avg_abs_diff_csv_path, beta, read_random, use_random)

    tt_avg_diff_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        tt_avg_diff_csv_path, beta, read_random, use_random)
    sd_avg_diff_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        sd_avg_diff_csv_path, beta, read_random, use_random)

    experiment_set.create_plot_dirs(beta, read_random, use_random)

    ### TT

    tt_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, tt_ybotlim,
                                                                tt_ytoplim)  # -3.3, 4.7)
    fig_tt, ax_tt = plt.subplots(figsize=tt_plot_parameters.FIG_SIZE)

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

    tt_df = get_elementwise_difference_df(tt_df_1, tt_df_2, mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, per_path=False, plot_params=tt_plot_parameters)

    # get series with for each discrete difference, the amount of time steps that have this difference, and the percentage of time steps that have this difference
    tt_diff_counts = tt_df["avg_travel_time"].apply(
        lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()

    save_value_count_table_as_csv(tt_diff_counts, tt_diff_sign_count_csv_path, val_col_name="Sign",
                                  percent_col_name="Percentage")
    # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
    plot_value_count_table(ax_tt, tt_diff_counts, val_col_name="Sign", plot_params=tt_plot_parameters)

    tt_avg_diff = tt_df["avg_travel_time"].sum() / tt_df["avg_travel_time"].count()
    np.savetxt(tt_avg_diff_csv_path, [tt_avg_diff], )

    tt_avg_abs_diff = tt_df["avg_travel_time"].abs().sum() / tt_df["avg_travel_time"].count()
    np.savetxt(tt_avg_abs_diff_csv_path, [tt_avg_abs_diff], )

    plot_textbox(ax_tt, f"Average difference: {tt_avg_diff:.4f}\nAverage absolute difference: {tt_avg_abs_diff:.4f}",
                 x=0.01, y=0.11, plot_params=tt_plot_parameters)

    fig_tt.savefig(experiment_set.get_tt_plot_path(beta, read_random, use_random))

    ### SD
    sd_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, sd_ybotlim,
                                                                sd_ytoplim)  # -1.0, 1.0)
    fig_sd, ax_sd = plt.subplots(figsize=sd_plot_parameters.FIG_SIZE)

    if "{seed}" in sd_path_1:
        sd_df_1 = get_elementwise_avg_df(sd_path_1, seeds_to_use=seeds_to_use, mode="sd")
    else:
        sd_df_1 = pd.read_csv(sd_path_1)

    if "{seed}" in sd_path_2:
        sd_df_2 = get_elementwise_avg_df(sd_path_2, seeds_to_use=seeds_to_use, mode="sd")
    else:
        sd_df_2 = pd.read_csv(sd_path_2)

    sd_df = get_elementwise_difference_df(sd_df_1, sd_df_2, mode="sd")

    plot_extracted_sd_over_time(ax_sd, sd_df, plot_params=sd_plot_parameters)

    # get series with for each discrete difference, the amount of time steps that have this difference
    sd_diff_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).value_counts().sort_index()

    save_value_count_table_as_csv(sd_diff_counts, sd_diff_sign_count_csv_path, val_col_name="Sign",
                                  percent_col_name="Percentage")

    sd_avg_diff = sd_df["sum_departures_total"].sum() / sd_df["sum_departures_total"].count()
    np.savetxt(sd_avg_diff_csv_path, [sd_avg_diff], )
    sd_avg_abs_diff = sd_df["sum_departures_total"].abs().sum() / sd_df["sum_departures_total"].count()
    np.savetxt(sd_avg_abs_diff_csv_path, [sd_avg_abs_diff], )

    plot_textbox(ax_sd, f"Average difference: {sd_avg_diff:.4f}\nAverage absolute difference: {sd_avg_abs_diff:.4f}",
                 x=0.01, y=0.11, plot_params=sd_plot_parameters)

    # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
    plot_value_count_table(ax_sd, sd_diff_counts, plot_params=sd_plot_parameters, val_col_name="Sign")

    fig_sd.savefig(experiment_set.get_sd_plot_path(beta, read_random, use_random))
