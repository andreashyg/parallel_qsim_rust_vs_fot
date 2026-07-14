import os
import sys
from typing import List

import pandas as pd
import numpy as np
from matplotlib import pyplot as plt

from python_plotting.braess.utils import get_interpolated_nash_vals
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

            # Mean across all paths and time steps
            mean_deviation = np.mean(deviations_over_time_per_path)
            result.loc[seed, beta] = mean_deviation

    # Convert to numeric type
    result = result.astype(float)
    return result


if __name__ == '__main__':
    _, replanning_variant, seeds_to_avg_over, output_dir = sys.argv

    if seeds_to_avg_over == "java":
        # common for both tt and sd .csv file
        file_name_end = (
            # the plots will average over beta and read_from_random
                "_beta{beta}_read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
                + "_use_random_seed_{1}.csv"  # we only use one random seed in rust
        )
    elif seeds_to_avg_over == "rust":
        # the plots will average over beta and use_random_seed
        file_name_end = "_beta{beta}_read_from_random_{1}_use_random_seed_{seed}"
    else:
        raise ValueError("Invalid value for 'which_seeds', must be 'java' or 'rust'")

    tt_path = ROOT_DATA_PATH + f"{replanning_variant}/analysis/average_route_tts_per_deptime" + file_name_end
    sd_path = ROOT_DATA_PATH + f"{replanning_variant}/analysis/summed_deps_per_time" + file_name_end

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    tt_df = compute_nash_deviation_df(tt_path, betas=BETAS, seeds=list(range(42, 62)), mode="tt")

    tt_df.plot(ax=ax_tt, kind="box")
    # plot_extracted_tt_over_time(ax_tt, tt_df, per_path=False)

    try:
        os.makedirs(output_dir + "/tt_avg_deviation_boxplots")
    except FileExistsError:
        pass

    fig_tt.savefig(
        output_dir + f"/tt_avg_deviation_boxplots/tt_avg_deviation_boxplot.png")

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    sd_df = compute_nash_deviation_df(sd_path, betas=BETAS, seeds=list(range(42, 62)), mode="sd")
    # plot_extracted_sd_over_time(ax_sd, sd_df)
    sd_df.plot(ax=ax_sd, kind="box")

    try:
        os.makedirs(output_dir + "/sd_avg_deviation_boxplots")
    except FileExistsError:
        pass

    fig_sd.savefig(
        output_dir + f"/sd_avg_deviation_boxplots/sd_avg_deviation_boxplot.png")
