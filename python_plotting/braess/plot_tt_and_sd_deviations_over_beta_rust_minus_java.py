import os
import sys

from matplotlib import pyplot as plt

from utils import plot_boxplot_over_beta, plot_scatter_over_beta, compute_deviation_to_reference_df, \
    compute_deviation_to_reference_series_but_avg_first
from setup import ROOT_DATA_PATH, FIG_SIZE, BETAS, RUST_SEEDS_TO_ITERATE_OVER, \
    JAVA_SEED_INDEX_WHEN_FIXED, BOXPLOT_FIG_SIZE

if __name__ == '__main__':
    _, replanning_variant, fixed_file_dir, dir_to_avg, output_dir = sys.argv

    if fixed_file_dir != "recreating_java_results":
        raise ValueError(f"fixed_file_dir must be 'recreating_java_results', but got {fixed_file_dir}")
    fixed_file_end = ("_beta{beta}_"
                      + f"read_from_random_{JAVA_SEED_INDEX_WHEN_FIXED}"
                      + "_reformatted_original_java_data.csv"
                      )
    if dir_to_avg == "recreating_java_results":
        raise ValueError(f"dir_to_avg must not be 'recreating_java_results', but got {dir_to_avg}")
    file_pattern_to_avg = ("_beta{beta}_" + f"read_from_random_{JAVA_SEED_INDEX_WHEN_FIXED}_use_random_seed_"
                           + "{seed}.csv"  # deliberately not an f-string, used as placeholder later
                           )

    tt_path_fixed = ROOT_DATA_PATH + f"/{replanning_variant}/{fixed_file_dir}/analysis/extracted_data/average_route_tts_per_deptime" + fixed_file_end
    sd_path_fixed = ROOT_DATA_PATH + f"/{replanning_variant}/{fixed_file_dir}/analysis/extracted_data/summed_deps_per_time" + fixed_file_end

    tt_path_to_avg = ROOT_DATA_PATH + f"/{replanning_variant}/{dir_to_avg}/analysis/extracted_data/average_route_tts_per_deptime" + file_pattern_to_avg
    sd_path_to_avg = ROOT_DATA_PATH + f"/{replanning_variant}/{dir_to_avg}/analysis/extracted_data/summed_deps_per_time" + file_pattern_to_avg

    seeds = RUST_SEEDS_TO_ITERATE_OVER  # use_random seeds for rust are 42..61
    fixed_seed_string_for_filename = "read_from_random"

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    tt_df = compute_deviation_to_reference_df(tt_path_to_avg, betas=BETAS, seeds=seeds, mode="tt",
                                              reference_values_path=tt_path_fixed)

    plot_boxplot_over_beta(tt_df, ax_tt, "tt")

    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=FIG_SIZE)
    tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path_to_avg, betas=BETAS, seeds=seeds,
                                                                        mode="tt",
                                                                        reference_values_path=tt_path_fixed)

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt")

    try:
        os.makedirs(output_dir + "/tt_avg_deviation_boxplots__rust_minus_java")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_avg_deviation_boxplots__rust_minus_java/tt_avg_deviation_boxplot.pdf")

    try:
        os.makedirs(output_dir + "/tt_avg_first_deviation_scatterplots__rust_minus_java")
    except FileExistsError:
        pass

    fig_tt_avg.savefig(
        output_dir + f"/tt_avg_first_deviation_scatterplots__rust_minus_java/tt_avg_first_deviation_scatterplot.pdf"
    )

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    sd_df = compute_deviation_to_reference_df(sd_path_to_avg, betas=BETAS, seeds=seeds, mode="sd",
                                              reference_values_path=sd_path_fixed)

    plot_boxplot_over_beta(sd_df, ax_sd, "sd")

    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=FIG_SIZE)
    sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path_to_avg, betas=BETAS, seeds=seeds,
                                                                        mode="sd",
                                                                        reference_values_path=sd_path_fixed)

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd")

    try:
        os.makedirs(output_dir + "/sd_avg_deviation_boxplots__rust_minus_java")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_avg_deviation_boxplots__rust_minus_java/sd_avg_deviation_boxplot.pdf")

    try:
        os.makedirs(output_dir + "/sd_avg_first_deviation_scatterplots__rust_minus_java")
    except FileExistsError:
        pass

    fig_sd_avg.savefig(
        output_dir + f"/sd_avg_first_deviation_scatterplots__rust_minus_java/sd_avg_first_deviation_scatterplot.pdf"
    )
