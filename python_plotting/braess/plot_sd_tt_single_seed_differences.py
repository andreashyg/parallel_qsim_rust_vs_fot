import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys

from setup import FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_extracted_sd_over_time, plot_extracted_tt_over_time, \
    get_elementwise_difference_df, plot_textbox, plot_value_count_table, ExperimentSet, save_value_count_table_as_csv

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/perSeed/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/perSeed/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}.pdf"

tt_diff_value_count_csv_path = "{common_plots_pattern}/perSeed/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}ValueCount.csv"
sd_diff_value_count_csv_path = "{common_plots_pattern}/perSeed/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}ValueCount.csv"
tt_diff_sign_count_csv_path = "{common_plots_pattern}/perSeed/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}SignCount.csv"
sd_diff_sign_count_csv_path = "{common_plots_pattern}/perSeed/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}SignCount.csv"
tt_total_diff_path = "{common_plots_pattern}/perSeed/ttPerPathOverDeptimeMinus{minus_what}/ttPerPathOverDeptimeMinus{minus_what}{file_name_end}AvgAbsDiff.csv"
sd_total_diff_path = "{common_plots_pattern}/perSeed/sdPerPathOverTimeMinus{minus_what}/sdPerPathOverTimeMinus{minus_what}{file_name_end}AvgAbsDiff.csv"

if __name__ == '__main__':
    (_, beta, replanning_variant, read_random, use_random, experiment_set_name, base_output_dir,
     minus_what, main_input_tt_csv_path_pattern,
     main_input_sd_csv_path_pattern, secondary_input_tt_csv_path_pattern,
     secondary_input_sd_csv_path_pattern) = sys.argv

    beta = int(beta)
    read_random = int(read_random)

    if use_random.lower() == "none":
        use_random = None
    else:
        use_random = int(use_random)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)

    # also replace the extra minus_what placeholder
    output_tt_plot_path_pattern = output_tt_plot_path_pattern.replace("{minus_what}", minus_what)
    output_sd_plot_path_pattern = output_sd_plot_path_pattern.replace("{minus_what}", minus_what)

    # same thing for the csv output file paths for the diff value counts
    tt_diff_value_count_csv_path = tt_diff_value_count_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    sd_diff_value_count_csv_path = sd_diff_value_count_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    tt_diff_value_count_csv_path = tt_diff_value_count_csv_path.replace("{minus_what}", minus_what)
    sd_diff_value_count_csv_path = sd_diff_value_count_csv_path.replace("{minus_what}", minus_what)

    # and same thing for the diff sign counts
    tt_diff_sign_count_csv_path = tt_diff_sign_count_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    sd_diff_sign_count_csv_path = sd_diff_sign_count_csv_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    tt_diff_sign_count_csv_path = tt_diff_sign_count_csv_path.replace("{minus_what}", minus_what)
    sd_diff_sign_count_csv_path = sd_diff_sign_count_csv_path.replace("{minus_what}", minus_what)

    # and same thing for the total diff
    tt_total_diff_csv_path = tt_total_diff_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    sd_total_diff_csv_path = sd_total_diff_path.replace("{common_plots_pattern}", COMMON_PLOTS_PATTERN)
    tt_total_diff_csv_path = tt_total_diff_csv_path.replace("{minus_what}", minus_what)
    sd_total_diff_csv_path = sd_total_diff_csv_path.replace("{minus_what}", minus_what)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern, output_sd_plot_path_pattern,
                                   main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
                                   secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern)

    tt_path_1 = experiment_set.get_path_to_tt_csv_to_read(beta, read_random, use_random, secondary=False)
    sd_path_1 = experiment_set.get_path_to_sd_csv_to_read(beta, read_random, use_random, secondary=False)
    tt_path_2 = experiment_set.get_path_to_tt_csv_to_read(beta, read_random, use_random, secondary=True)
    sd_path_2 = experiment_set.get_path_to_sd_csv_to_read(beta, read_random, use_random, secondary=True)

    # replace beta, seeds etc in csv file patterns:
    tt_diff_value_count_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        tt_diff_value_count_csv_path, beta, read_random, use_random)
    sd_diff_value_count_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(
        sd_diff_value_count_csv_path, beta, read_random, use_random)
    tt_diff_sign_count_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(tt_diff_sign_count_csv_path,
                                                                                           beta, read_random,
                                                                                           use_random)
    sd_diff_sign_count_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(sd_diff_sign_count_csv_path,
                                                                                           beta, read_random,
                                                                                           use_random)
    tt_total_diff_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(tt_total_diff_csv_path, beta,
                                                                                      read_random, use_random)
    sd_total_diff_csv_path = experiment_set.replace_placeholders_in_arbitrary_pattern(sd_total_diff_csv_path, beta,
                                                                                      read_random, use_random)

    experiment_set.create_plot_dirs(beta, read_random, use_random)

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    # read tt data
    tt_df = get_elementwise_difference_df(pd.read_csv(tt_path_1), pd.read_csv(tt_path_2), mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, ybotlim=-3.3, ytoplim=4.7)

    # get series with for each discrete difference, the amount of time steps that have this difference
    tt_concated_values = pd.concat([tt_df[f'avg_travel_time_path_{i}'] for i in range(3)])
    tt_diff_counts = tt_concated_values.apply(
        lambda x: round(x, 4)).value_counts().sort_index()
    # save value counts into a csv
    save_value_count_table_as_csv(tt_diff_counts, tt_diff_value_count_csv_path)

    # get the count of negative, positive and zero differences, and save it into a csv
    tt_diff_sign_counts = pd.concat([tt_df[f'avg_travel_time_path_{i}'] for i in range(3)]).apply(
        lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()

    save_value_count_table_as_csv(tt_diff_sign_counts, tt_diff_sign_count_csv_path, val_col_name="Sign",
                                  percent_col_name="Percentage")

    # The total difference is the average absolute difference over all time steps, of the avg travel time over all paths
    # tt_total_diff = tt_concated_values.abs().sum() / tt_concated_values.count()
    tt_total_diff = tt_df['avg_travel_time'].abs().sum() / tt_df['avg_travel_time'].count()
    np.savetxt(tt_total_diff_csv_path, [tt_total_diff], fmt="%.5f")

    plot_textbox(ax_tt, f"Average absolute difference: {tt_total_diff:.4f}", x=0.01, y=0.1)

    # If there are too many different values, we will only show the sign of the difference in the table in the plot
    if tt_diff_counts.size > 7:
        plot_value_count_table(ax_tt, tt_diff_sign_counts, val_col_name="Sign")
    else:
        # include this as a table in the plot, with the difference in the first column, the count in the second column
        plot_value_count_table(ax_tt, tt_diff_counts)

    fig_tt.savefig(experiment_set.get_tt_plot_path(beta, read_random, use_random))

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    # read sd data
    sd_df = get_elementwise_difference_df(pd.read_csv(sd_path_1), pd.read_csv(sd_path_2), mode="sd")

    plot_extracted_sd_over_time(ax_sd, sd_df, ybotlim=-1, ytoplim=1)

    # get series with for each discrete difference, the amount of time steps that have this difference
    sd_diff_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).apply(
        lambda x: round(x, 4)).value_counts().sort_index()

    # save value counts into a csv
    save_value_count_table_as_csv(sd_diff_counts, sd_diff_value_count_csv_path)

    # get the count of negative, positive and zero differences, and save it into a csv
    sd_diff_sign_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).apply(
        lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()

    save_value_count_table_as_csv(sd_diff_sign_counts, sd_diff_sign_count_csv_path, val_col_name="Sign",
                                  percent_col_name="Percentage")

    # The total difference is the average absolute difference over all time steps, of the avg travel time over all paths
    # tt_total_diff = tt_concated_values.abs().sum() / tt_concated_values.count()
    sd_total_diff = sd_df['sum_departures_total'].abs().sum() / sd_df['sum_departures_total'].count()
    np.savetxt(sd_total_diff_csv_path, [sd_total_diff], fmt="%.5f")

    plot_textbox(ax_sd, f"Average absolute difference: {sd_total_diff:.4f}", x=0.01, y=0.1)

    if sd_diff_counts.size > 7:
        # if there are too many different values, we will only show the sign of the difference in the table in the plot
        plot_value_count_table(ax_sd, sd_diff_sign_counts, val_col_name="Sign")
    else:
        # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
        plot_value_count_table(ax_sd, sd_diff_counts)

    fig_sd.savefig(experiment_set.get_sd_plot_path(beta, read_random, use_random))
