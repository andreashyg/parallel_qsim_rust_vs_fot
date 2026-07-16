import os
import sys
from typing import List

import pandas as pd
import numpy as np
from matplotlib import pyplot as plt

from utils import get_interpolated_nash_vals, plot_boxplot_over_beta, get_elementwise_avg_df, plot_scatter_over_beta
from setup import ROOT_DATA_PATH, FIG_SIZE, BETAS


def compute_nash_deviation_df(csv_path_template: str, betas: List, seeds: List[int], mode: str) -> pd.DataFrame:
    """
    Load CSV files for different beta and seed combinations, compute the mean absolute
    deviation from Nash equilibrium across all time steps and paths.

    # TODO should maybe change:
    **NOTE:** currently, seeds are used as parameter for "use_random_seeds", but in the future, it should also be able
    to be used for "read_random_seeds"

    # TODO update text if calculation is changed.
    Currently, the absolute value is taken at the lowest level, i.e., when calculating the deviation between e.g.
    sd from the nash reference value **for a specific path, at a specific time, for a specific seed and specific beta**.
    Then the mean of those absolute deviations is taken across all paths and time steps, for that specific seed-beta
    combination.
    
    Returns DataFrame with shape (seeds, betas), where each cell contains the average
    absolute deviation from Nash values for that seed-beta combination.
    """

    if mode == "sd":
        index_col = "time"
        data_col = "sum_departures_path_{i}"
    elif mode == "tt":
        index_col = "departure_time"
        data_col = "avg_travel_time_path_{i}"
    else:
        raise ValueError("Mode must be either 'tt' or 'sd'")

    # Initialize result DataFrame with seeds as index and betas as columns
    result = pd.DataFrame(index=seeds, columns=betas)

    # for all betas and all seeds
    for beta in betas:
        for seed in seeds:
            # load the csv
            path = csv_path_template.format(seed=seed, beta=beta)
            try:
                df = pd.read_csv(path)
                # print("successfully read df:")
                # print(df)
            except FileNotFoundError as e:
                print(f"Warning: Failed to read '{path}': {e}")
                result.loc[seed, beta] = np.nan
                continue

            # try to set the index depending on the mode
            if index_col not in df.columns:
                raise ValueError(f"'{index_col}' column not found in '{path}'")

            df = df.set_index(index_col)

            # Compute deviations for all 3 paths
            deviations_over_time_per_path = []
            for path_idx in range(3):  # can be hard-coded, braess always has 3 paths
                col_name = data_col.format(i=path_idx)
                if col_name not in df.columns:
                    raise ValueError(f"'{col_name}' not found in '{path}'")

                # Calculate reference values (from Nash values, between which the function is linear)
                nash_values = get_interpolated_nash_vals(df.index, mode, for_path=path_idx)

                # TODO this is where something would have to change, if the absolute value is taken at another place.
                # Calculate absolute deviation of sd or tt w.r.t. the nash reference value
                # -> the result is a vector with the deviation per time step
                # (note: we are still in the case of per seed, per beta here)
                deviation_over_time_at_path_i = np.abs(df[col_name].values - nash_values)
                deviations_over_time_per_path.append(deviation_over_time_at_path_i)

            # print(deviations_over_time_per_path)
            # Mean across all paths and time steps
            mean_deviation = np.nanmean(deviations_over_time_per_path)
            result.loc[seed, beta] = mean_deviation

    # Convert to numeric type
    result = result.astype(float)
    # print(result)
    return result


def compute_nash_deviation_series_but_avg_first(csv_path_template: str, betas: List, seeds: List[int],
                                                mode: str) -> pd.Series:
    """
    #TODO docstring is copy pasted, needs update
    Load CSV files for different beta and seed combinations, compute the mean absolute
    deviation from Nash equilibrium across all time steps and paths.

    # TODO should maybe change:
    **NOTE:** currently, seeds are used as parameter for "use_random_seeds", but in the future, it should also be able
    to be used for "read_random_seeds"

    # TODO update text if calculation is changed.
    Currently, the absolute value is taken at the lowest level, i.e., when calculating the deviation between e.g.
    sd from the nash reference value **for a specific path, at a specific time, for a specific seed and specific beta**.
    Then the mean of those absolute deviations is taken across all paths and time steps, for that specific seed-beta
    combination.

    Returns DataFrame with shape (seeds, betas), where each cell contains the average
    absolute deviation from Nash values for that seed-beta combination.
    """

    if mode == "sd":
        index_col = "time"
        data_col = "sum_departures_path_{i}"
    elif mode == "tt":
        index_col = "departure_time"
        data_col = "avg_travel_time_path_{i}"
    else:
        raise ValueError("Mode must be either 'tt' or 'sd'")

    # Initialize result DataFrame with seeds as index and betas as columns
    result = pd.Series(index=betas)

    # for all betas and all seeds
    for beta in betas:
        csv_path_template_with_fixed_beta = csv_path_template.replace("{beta}", str(beta))
        averaged_df = get_elementwise_avg_df(csv_path_template_with_fixed_beta, seeds, mode)

        # try to set the index depending on the mode
        if index_col not in averaged_df.columns:
            raise ValueError(f"'{index_col}' column not found in the averaged dataframe")

        averaged_df = averaged_df.set_index(index_col)
        # print(averaged_df)

        # Compute deviations for all 3 paths
        deviations_over_time_per_path = []
        for path_idx in range(3):  # can be hard-coded, braess always has 3 paths

            col_name = data_col.format(i=path_idx)
            if col_name not in averaged_df.columns:
                raise ValueError(f"'{col_name}' not found in the averaged dataframe")

            # Calculate reference values (from Nash values, between which the function is linear)
            nash_values = get_interpolated_nash_vals(averaged_df.index, mode, for_path=path_idx)

            # TODO this is where something would have to change, if the absolute value is taken at another place.
            # Calculate absolute deviation of sd or tt w.r.t. the nash reference value
            # -> the result is a vector with the deviation per time step
            # (note: we are still in the case of per seed, per beta here)

            # uncomment this to first average over time (for fixed path), then take abs value, then avg over paths
            # deviation_over_time_at_path_i_avgd = np.abs(np.nanmean(averaged_df[col_name].values - nash_values))
            # deviations_over_time_per_path.append(deviation_over_time_at_path_i_avgd)

            deviation_over_time_at_path_i = np.abs(averaged_df[col_name].values - nash_values)
            deviations_over_time_per_path.append(deviation_over_time_at_path_i)

        # print(deviations_over_time_per_path)
        # Mean across all paths and time steps
        mean_deviation = np.nanmean(deviations_over_time_per_path)
        result.loc[beta] = mean_deviation

    # Convert to numeric type
    result = result.astype(float)
    # print(result)
    return result


if __name__ == '__main__':
    _, replanning_variant, seeds_to_avg_over, output_dir = sys.argv

    if seeds_to_avg_over == "java":
        # common for both tt and sd .csv file
        file_name_end = (
            # the plots will average over beta and read_from_random
                "_beta{beta}_read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
                + "_use_random_seed_42.csv"  # we only use one random seed in rust
        )
        seeds = list(range(1, 21))  # read_from_random seeds are 1..20
    elif seeds_to_avg_over == "rust":
        # the plots will average over beta and use_random_seed
        file_name_end = "_beta{beta}_read_from_random_1_use_random_seed_{seed}.csv"
        seeds = list(range(42, 62))  # use_random_seeds are 42..61
    else:
        raise ValueError("Invalid value for 'which_seeds', must be 'java' or 'rust'")

    tt_path = ROOT_DATA_PATH + f"{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    sd_path = ROOT_DATA_PATH + f"{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    tt_df = compute_nash_deviation_df(tt_path, betas=BETAS, seeds=seeds, mode="tt")

    plot_boxplot_over_beta(tt_df, ax_tt, "tt")

    fig_tt_avg, ax_tt_avg = plt.subplots(figsize=FIG_SIZE)
    tt_avg_series = compute_nash_deviation_series_but_avg_first(tt_path, betas=BETAS, seeds=seeds, mode="tt")

    plot_scatter_over_beta(tt_avg_series, ax_tt_avg, "tt")

    try:
        os.makedirs(output_dir + "/tt_avg_deviation_boxplots")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_avg_deviation_boxplots/tt_avg_deviation_boxplot.png")

    try:
        os.makedirs(output_dir + "/tt_avg_first_deviation_scatterplots")
    except FileExistsError:
        pass

    fig_tt_avg.savefig(
        output_dir + f"/tt_avg_first_deviation_scatterplots/tt_avg_first_deviation_scatterplot.png"
    )

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    sd_df = compute_nash_deviation_df(sd_path, betas=BETAS, seeds=seeds, mode="sd")

    plot_boxplot_over_beta(sd_df, ax_sd, "sd")

    fig_sd_avg, ax_sd_avg = plt.subplots(figsize=FIG_SIZE)
    sd_avg_series = compute_nash_deviation_series_but_avg_first(sd_path, betas=BETAS, seeds=seeds, mode="sd")

    plot_scatter_over_beta(sd_avg_series, ax_sd_avg, "sd")

    try:
        os.makedirs(output_dir + "/sd_avg_deviation_boxplots")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_avg_deviation_boxplots/sd_avg_deviation_boxplot.png")

    try:
        os.makedirs(output_dir + "/sd_avg_first_deviation_scatterplots")
    except FileExistsError:
        pass

    fig_sd_avg.savefig(
        output_dir + f"/sd_avg_first_deviation_scatterplots/sd_avg_first_deviation_scatterplot.png"
    )
