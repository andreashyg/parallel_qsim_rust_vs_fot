import os
import sys

import pandas as pd
import matplotlib.pyplot as plt

from setup import FIG_SIZE, ROOT_DATA_PATH
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time

if __name__ == '__main__':
    _, beta, replanning_variant, read_random, use_random, seeds_to_avg_over, read_original_java, output_dir = sys.argv

    if read_original_java.lower() == "true":
        read_original_java = True
    else:
        read_original_java = False

    if read_original_java:
        file_name_end = f"_beta{beta}_read_from_random_{read_random}_reformatted_original_java_data.csv"
        tt_path = ROOT_DATA_PATH + f"{replanning_variant}/recreating_java_results/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
        sd_path = ROOT_DATA_PATH + f"{replanning_variant}/recreating_java_results/analysis/extracted_data/summed_deps_per_time" + file_name_end

    else:
        # common for both tt and sd .csv file
        file_name_end = f"_beta{beta}_read_from_random_{read_random}_use_random_seed_{use_random}.csv"
        tt_path = ROOT_DATA_PATH + f"{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
        sd_path = ROOT_DATA_PATH + f"{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_tt, mode="tt")
    # read tt data
    tt_df = pd.read_csv(tt_path)
    plot_extracted_tt_over_time(ax_tt, tt_df)

    try:
        os.makedirs(output_dir + "/tt_per_path_over_deptime")
    except FileExistsError:
        pass

    if read_original_java:
        fig_tt.savefig(
            output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_random}_original_java_data.png")
    else:
        fig_tt.savefig(
            output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_read_from_random_{read_random}_use_random_seed{use_random}.png")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_sd, mode="sd")
    # read sd data
    sd_df = pd.read_csv(sd_path)

    plot_extracted_sd_over_time(ax_sd, sd_df)

    try:
        os.makedirs(output_dir + "/sd_per_path_over_time")
    except FileExistsError:
        pass

    if read_original_java:
        fig_sd.savefig(
            output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_random}_original_java_data.png")
    else:
        fig_sd.savefig(
            output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_read_from_random_{read_random}_use_random_seed{use_random}.png")
