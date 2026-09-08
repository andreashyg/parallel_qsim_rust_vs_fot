import os

import sys
from matplotlib import pyplot as plt

from setup import ROOT_DATA_PATH, FIG_SIZE, BETAS, JAVA_SEED_INDICES_TO_ITERATE_OVER, RUST_SEEDS_TO_ITERATE_OVER, \
    RUST_SEED_WHEN_FIXED, JAVA_SEED_INDEX_WHEN_FIXED, BOXPLOT_FIG_SIZE
from utils import plot_boxplot_over_beta, plot_scatter_over_beta, \
    compute_deviation_to_reference_df, compute_deviation_to_reference_series_but_avg_first

if __name__ == '__main__':
    _, replanning_variant, seeds_to_avg_over, output_dir, read_original_java = sys.argv

    if read_original_java.lower() == "true":
        read_original_java = True
    else:
        read_original_java = False

    if seeds_to_avg_over == "java":
        if read_original_java:
            # common for both tt and sd .csv file
            file_name_end = (
                # the plots will average over beta and read_from_random
                    "_beta{beta}_read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
                    + "_reformatted_original_java_data.csv"  # we only use one random seed in rust
            )
            tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/recreating_java_results/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
            sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/recreating_java_results/analysis/extracted_data/summed_deps_per_time" + file_name_end

        else:
            # common for both tt and sd .csv file
            file_name_end = (
                # the plots will average over beta and read_from_random
                    "_beta{beta}_read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
                    + f"_use_random_seed_{RUST_SEED_WHEN_FIXED}.csv"  # we only use one random seed in rust
            )
            tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
            sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end

        seeds = JAVA_SEED_INDICES_TO_ITERATE_OVER  # read_from_random seeds are 1..20
    elif seeds_to_avg_over == "rust":
        if read_original_java:
            raise ValueError(
                "Parameter combination read_original_java=True and seeds_to_avg_over=rust is not supported")

        # the plots will average over beta and use_random_seed (deliberately only one of the strings is f-string)
        file_name_end = "_beta{beta}" + f"_read_from_random_{JAVA_SEED_INDEX_WHEN_FIXED}" + "_use_random_seed_{seed}.csv"

        tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
        sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end

        seeds = RUST_SEEDS_TO_ITERATE_OVER  # use_random_seeds are 42..61
    else:
        raise ValueError("Invalid value for 'which_seeds', must be 'java' or 'rust'")

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    tt_df = compute_deviation_to_reference_df(tt_path, betas=BETAS, seeds=seeds, mode="tt")

    plot_boxplot_over_beta(tt_df, ax_tt, "tt")

    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=FIG_SIZE)
    tt_avg_series = compute_deviation_to_reference_series_but_avg_first(tt_path, betas=BETAS, seeds=seeds, mode="tt")

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt")

    try:
        os.makedirs(output_dir + "/tt_avg_deviation_boxplots")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_avg_deviation_boxplots/tt_avg_deviation_boxplot.pdf")

    try:
        os.makedirs(output_dir + "/tt_avg_first_deviation_scatterplots")
    except FileExistsError:
        pass

    fig_tt_avg.savefig(
        output_dir + f"/tt_avg_first_deviation_scatterplots/tt_avg_first_deviation_scatterplot.pdf"
    )

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=BOXPLOT_FIG_SIZE)
    sd_df = compute_deviation_to_reference_df(sd_path, betas=BETAS, seeds=seeds, mode="sd")

    plot_boxplot_over_beta(sd_df, ax_sd, "sd")

    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=FIG_SIZE)
    sd_avg_series = compute_deviation_to_reference_series_but_avg_first(sd_path, betas=BETAS, seeds=seeds, mode="sd")

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd")

    try:
        os.makedirs(output_dir + "/sd_avg_deviation_boxplots")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_avg_deviation_boxplots/sd_avg_deviation_boxplot.pdf")

    try:
        os.makedirs(output_dir + "/sd_avg_first_deviation_scatterplots")
    except FileExistsError:
        pass

    fig_sd_avg.savefig(
        output_dir + f"/sd_avg_first_deviation_scatterplots/sd_avg_first_deviation_scatterplot.pdf"
    )
