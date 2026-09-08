import os
import sys

import pandas as pd
import matplotlib.pyplot as plt

from setup import FIG_SIZE, ROOT_DATA_PATH
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time, \
    get_elementwise_difference_df, plot_textbox, plot_value_count_table

if __name__ == '__main__':
    _, beta, replanning_variant, read_random, use_random, main_dir_1, main_dir_2, output_dir = sys.argv

    if main_dir_1 == "recreating_java_results":
        file_name_end_1 = f"_beta{beta}_read_from_random_{read_random}_reformatted_original_java_data.csv"
    else:
        file_name_end_1 = f"_beta{beta}_read_from_random_{read_random}_use_random_seed_{use_random}.csv"

    if main_dir_2 == "recreating_java_results":
        file_name_end_2 = f"_beta{beta}_read_from_random_{read_random}_reformatted_original_java_data.csv"
    else:
        file_name_end_2 = f"_beta{beta}_read_from_random_{read_random}_use_random_seed_{use_random}.csv"

    tt_path_1 = ROOT_DATA_PATH + f"/{replanning_variant}/{main_dir_1}/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end_1
    sd_path_1 = ROOT_DATA_PATH + f"/{replanning_variant}/{main_dir_1}/analysis/extracted_data/summed_deps_per_time" + file_name_end_1

    tt_path_2 = ROOT_DATA_PATH + f"/{replanning_variant}/{main_dir_2}/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end_2
    sd_path_2 = ROOT_DATA_PATH + f"/{replanning_variant}/{main_dir_2}/analysis/extracted_data/summed_deps_per_time" + file_name_end_2

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    # read tt data
    tt_df = get_elementwise_difference_df(pd.read_csv(tt_path_1), pd.read_csv(tt_path_2), mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, ybotlim=-3.3, ytoplim=4.7)

    # get series with for each discrete difference, the amount of time steps that have this difference
    tt_concated_values = pd.concat([tt_df[f'avg_travel_time_path_{i}'] for i in range(3)])
    tt_diff_counts = tt_concated_values.apply(
        lambda x: round(x, 4)).value_counts().sort_index()

    # tt_total_diff = tt_concated_values.abs().sum() / tt_concated_values.count()
    tt_total_diff = tt_df['avg_travel_time'].abs().sum() / tt_df['avg_travel_time'].count()

    plot_textbox(ax_tt, f"Average absolute difference: {tt_total_diff:.4f}", x=0.01, y=0.1)

    # If there are too many different values, we will only show the sign of the difference in the table
    if tt_diff_counts.size > 7:
        tt_diff_counts = pd.concat([tt_df[f'avg_travel_time_path_{i}'] for i in range(3)]).apply(
            lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()
        plot_value_count_table(ax_tt, tt_diff_counts, val_col_name="Sign")
    else:
        # include this as a table in the plot, with the difference in the first column, the count in the second column
        plot_value_count_table(ax_tt, tt_diff_counts)

    try:
        os.makedirs(output_dir + f"/tt_per_path_over_deptime__{main_dir_1}_minus_{main_dir_2}")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_per_path_over_deptime__{main_dir_1}_minus_{main_dir_2}/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_random}_use_random_seed{use_random}.pdf")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    # read sd data
    sd_df = get_elementwise_difference_df(pd.read_csv(sd_path_1), pd.read_csv(sd_path_2), mode="sd")

    plot_extracted_sd_over_time(ax_sd, sd_df, ybotlim=-1, ytoplim=1)

    # get series with for each discrete difference, the amount of time steps that have this difference
    sd_diff_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).apply(
        lambda x: round(x, 4)).value_counts().sort_index()

    if sd_diff_counts.size > 7:
        sd_diff_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).apply(
            lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()
        plot_value_count_table(ax_sd, sd_diff_counts, val_col_name="Sign")
    else:
        # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
        plot_value_count_table(ax_sd, sd_diff_counts)

    try:
        os.makedirs(output_dir + f"/sd_per_path_over_time__{main_dir_1}_minus_{main_dir_2}")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_per_path_over_time__{main_dir_1}_minus_{main_dir_2}/sd_per_path_over_time_beta{beta}_read_from_random_{read_random}_use_random_seed{use_random}.pdf")
