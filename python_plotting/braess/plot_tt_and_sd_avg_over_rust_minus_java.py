import os
import sys

import matplotlib.pyplot as plt

import pandas as pd
from pandas import IntervalIndex

from setup import FIG_SIZE, ROOT_DATA_PATH, RUST_SEEDS_TO_ITERATE_OVER
from utils import plot_extracted_sd_over_time, plot_extracted_tt_over_time, get_elementwise_avg_df, \
    get_elementwise_difference_df, plot_value_count_table, plot_textbox

if __name__ == '__main__':

    _, beta, replanning_variant, fixed_seed, fixed_file_dir, dir_to_avg, output_dir = sys.argv

    if fixed_file_dir == "recreating_java_results":
        fixed_file_end = (f"_beta{beta}_"
                          + f"read_from_random_{fixed_seed}"
                          + f"_reformatted_original_java_data.csv"
                          )
    else:
        raise ValueError(f"fixed_file_dir must be 'recreating_java_results', but got {fixed_file_dir}")

    if dir_to_avg == "recreating_java_results":
        raise ValueError(f"dir_to_avg must not be 'recreating_java_results', but got {dir_to_avg}")

    else:
        file_pattern_to_avg = (f"_beta{beta}_read_from_random_{fixed_seed}_use_random_seed_"
                               + "{seed}.csv"  # deliberately not an f-string, used as placeholder later
                               )

    tt_path_fixed = ROOT_DATA_PATH + f"/{replanning_variant}/{fixed_file_dir}/analysis/extracted_data/average_route_tts_per_deptime" + fixed_file_end
    sd_path_fixed = ROOT_DATA_PATH + f"/{replanning_variant}/{fixed_file_dir}/analysis/extracted_data/summed_deps_per_time" + fixed_file_end

    tt_path_to_avg = ROOT_DATA_PATH + f"/{replanning_variant}/{dir_to_avg}/analysis/extracted_data/average_route_tts_per_deptime" + file_pattern_to_avg
    sd_path_to_avg = ROOT_DATA_PATH + f"/{replanning_variant}/{dir_to_avg}/analysis/extracted_data/summed_deps_per_time" + file_pattern_to_avg

    seeds = RUST_SEEDS_TO_ITERATE_OVER  # use_random seeds for rust are 42..61
    fixed_seed_string_for_filename = "read_from_random"

    ### TT

    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    avg_tt_df = get_elementwise_avg_df(tt_path_to_avg, use_random_seeds=list(seeds), mode="tt")
    # get difference of averaged travel times over rust seeds and the fixed original java travel times
    tt_df = get_elementwise_difference_df(avg_tt_df, pd.read_csv(tt_path_fixed), mode="tt")
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

    try:
        os.makedirs(output_dir + "/tt_per_path_over_deptime__avg_over_rust_minus_java")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_per_path_over_deptime__avg_over_rust_minus_java/tt_per_path_over_deptime_beta{beta}_{fixed_seed_string_for_filename}_{fixed_seed}.pdf")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    avg_sd_df = get_elementwise_avg_df(sd_path_to_avg, use_random_seeds=seeds, mode="sd")
    # get difference of averaged summed departures over rust seeds and the fixed original java summed departures
    sd_df = get_elementwise_difference_df(avg_sd_df, pd.read_csv(sd_path_fixed), mode="sd")

    plot_extracted_sd_over_time(ax_sd, sd_df, ybotlim=-1, ytoplim=1)

    # get series with for each discrete difference, the amount of time steps that have this difference
    sd_diff_counts = pd.concat([sd_df[f'sum_departures_path_{i}'] for i in range(3)]).value_counts().sort_index()
    # sd_diff_counts = sd_df["sum_departures_total"].apply(
    #     lambda x: "<0" if x < 0 else ">0" if x > 0 else "0").value_counts().sort_index()

    # include this as a table in the plot, with the difference in the first column, the count in the second column, and the percentage in the third column
    plot_value_count_table(ax_sd, sd_diff_counts)

    try:
        os.makedirs(output_dir + "/sd_per_path_over_time__avg_over_rust_minus_java")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_per_path_over_time__avg_over_rust_minus_java/sd_per_path_over_time_beta{beta}_{fixed_seed_string_for_filename}_{fixed_seed}.pdf")
