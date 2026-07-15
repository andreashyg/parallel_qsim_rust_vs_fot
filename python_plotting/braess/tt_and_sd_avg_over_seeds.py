import os
import sys

import matplotlib.pyplot as plt

from setup import FIG_SIZE, ROOT_DATA_PATH
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time, get_elementwise_avg_df

if __name__ == '__main__':
    # note: fixed_seed can be either a read_from_random seed or a use_random_seed, depending on seeds_to_avg_over:
    #   - if seeds_to_avg_over == "java", we fix a rust seed, i.e., fixed_seed is a use_random_seed value (for example,
    #       we take use_random_seed=1 and average over all valued of read_from_random (that is, 1..20)
    #   - if seeds_to_avg_over == "rust", we fox a java seed, i.e., fixed_seed is a read_from_random value
    _, beta, replanning_variant, seeds_to_avg_over, fixed_seed, output_dir = sys.argv

    if seeds_to_avg_over == "java":
        # fix a use_random_seed value, but make read_from_random a placeholder to be formatted

        # common for both tt and sd .csv file
        file_name_end = (f"_beta{beta}_"
                         + "read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
                         + f"_use_random_seed_{fixed_seed}.csv"
                         )

        seeds = list(range(1, 21))  # read_from_random seeds for java are 1..20

    elif seeds_to_avg_over == "rust":
        # fix a read_from_random value, but make use_random_seed a placeholder to be formatted

        # common for both tt and sd .csv file
        file_name_end = (f"_beta{beta}_read_from_random_{fixed_seed}_use_random_seed_"
                         + "{seed}.csv"  # deliberately not an f-string, used as placeholder later
                         )

        seeds = list(range(42, 62))  # use_random seeds for rust are 42..61
    else:
        raise ValueError("Seeds to average over must be either 'java' or 'rust'")
    tt_path = ROOT_DATA_PATH + f"{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    sd_path = ROOT_DATA_PATH + f"{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_tt, mode="tt")
    tt_df = get_elementwise_avg_df(tt_path, use_random_seeds=list(seeds), mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, per_path=False)

    try:
        os.makedirs(output_dir + "/tt_per_path_over_deptime")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{fixed_seed}.png")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_sd, mode="sd")
    sd_df = get_elementwise_avg_df(sd_path, use_random_seeds=seeds, mode="sd")
    plot_extracted_sd_over_time(ax_sd, sd_df)

    try:
        os.makedirs(output_dir + "/sd_per_path_over_time")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{fixed_seed}.png")
