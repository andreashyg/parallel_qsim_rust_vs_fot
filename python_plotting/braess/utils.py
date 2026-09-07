from typing import Optional, List

import numpy as np
import pandas as pd
from matplotlib import pyplot as plt

from setup import NASH_TT_POINTS, NASH_SD_POINTS, COLORS, LABELS, FONT_SIZE, LEGEND_FONT_SIZE, \
    TOP_COLOUR, BETAS


def get_interpolated_nash_vals(at_xvals: pd.Index, mode: str, for_path: Optional[int] = None) -> np.ndarray:
    """
    Returns an array with the interpolated values of NASH_TT_POINTS or NASH_SD_POINTS, at the provided x values.
    """
    if mode == "tt":
        return np.interp(at_xvals, NASH_TT_POINTS[:, 0], NASH_TT_POINTS[:, 1])
    elif mode == "sd":
        if for_path is None:
            raise ValueError("for_path must be provided in mode sd")
        return np.interp(
            at_xvals,
            NASH_SD_POINTS[for_path, :, 0],
            NASH_SD_POINTS[for_path, :, 1]
        )
    else:
        raise ValueError("Mode must be either tt or sd")


def plot_nash_lines(ax: plt.Axes, mode: str):
    if mode == "tt":
        ax.plot(NASH_TT_POINTS[:, 0], NASH_TT_POINTS[:, 1], color="black", label="Nash Flow")
    elif mode == "sd":
        for i in range(NASH_SD_POINTS.shape[0]):
            ax.plot(NASH_SD_POINTS[i, :, 0], NASH_SD_POINTS[i, :, 1], color=COLORS[i])
    else:
        raise ValueError("Mode must be either tt or sd")


def plot_extracted_sd_over_time(ax: plt.Axes, sd_df: pd.DataFrame, ybotlim: Optional[float] = None,
                                ytoplim: Optional[float] = None):
    if ybotlim is None:
        ybotlim = 0  # default value if not provided
    if ytoplim is None:
        ytoplim = 45  # default value if not provided

    sd_df_copy = sd_df.copy()

    sd_df_copy["time"] = pd.concat(
        [sd_df_copy["time"], pd.Series([100])], ignore_index=True)  # append a 100 to the time column for plotting
    for i in range(3):
        # TODO is this what we want? It isn't done this way in the TT plots, but that probably makes sense because the TT plots aren't cumulative, but the SD plots are cumulative.
        sd_df_copy[f"sum_departures_path_{i}"] = pd.concat([
            pd.Series([0]),
            # prepend a 0 to the cumulative flow to make it start at 0 instead of 1, for better comparison with NASH lines
            sd_df_copy[f"sum_departures_path_{i}"]], ignore_index=True)

        sd_df_copy.plot(x="time", y=f"sum_departures_path_{i}", kind="scatter", ax=ax, label=LABELS[i], color=COLORS[i],
                        s=50, alpha=0.5)

    ax.set_xlabel("time [s]", fontsize=FONT_SIZE)
    ax.set_ylabel("cumulative flow [vol]", fontsize=FONT_SIZE)
    ax.xaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.yaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.set_xlim(left=0, right=100)
    ax.set_ylim(bottom=ybotlim, top=ytoplim)
    ax.grid(True)
    ax.legend(fontsize=LEGEND_FONT_SIZE)


def plot_extracted_tt_over_time(ax: plt.Axes, tt_df: pd.DataFrame, per_path: bool = True,
                                ybotlim: Optional[float] = None,
                                ytoplim: Optional[float] = None):
    if ybotlim is None:
        ybotlim = 0  # default value if not provided
    if ytoplim is None:
        ytoplim = 110  # default value if not provided
    if per_path:

        markers = ['o', 's', '^']  # Different markers for each path
        for i in range(3):
            tt_df.plot(x="departure_time", y=f"avg_travel_time_path_{i}", kind="scatter", ax=ax, label=LABELS[i],
                       color=COLORS[i], s=50, alpha=0.5, marker=markers[i])
    else:
        # tt_df_copy = tt_df.copy()
        # tt_df_copy["avg_travel_time"] = np.nanmean(
        #     [tt_df_copy["avg_travel_time_path_0"], tt_df_copy["avg_travel_time_path_1"],
        #      tt_df_copy["avg_travel_time_path_2"]], axis=0)
        # tt_df_copy.plot(x="departure_time", y="avg_travel_time", kind="scatter", ax=ax, label="avg. path",
        #                 color=TOP_COLOUR,
        #                 s=50, alpha=0.5)

        tt_df.plot(x="departure_time", y="avg_travel_time", kind="scatter", ax=ax, label="avg. path", color=TOP_COLOUR,
                   s=50, alpha=0.5)

    ax.set_xlabel("departure time [s]", fontsize=FONT_SIZE)
    ax.set_ylabel("travel time [s]", fontsize=FONT_SIZE)
    ax.xaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.yaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.set_xlim(left=0, right=100)
    ax.set_ylim(bottom=ybotlim, top=ytoplim)
    ax.grid(True)
    ax.legend(fontsize=LEGEND_FONT_SIZE)


def plot_boxplot_over_beta(df: pd.DataFrame, ax: plt.Axes, mode: str):
    if mode == "tt":
        ylabel = "avg. travel time deviation [s]"
    elif mode == "sd":
        ylabel = "cumulative flow deviation [vol]"
    else:
        raise ValueError("Mode must be either tt or sd")

    bplot = ax.boxplot(x=df,
                       patch_artist=True)
    ax.set_xlabel("time step size [s]", fontsize=FONT_SIZE)
    ax.set_ylabel(ylabel, fontsize=FONT_SIZE)
    ax.xaxis.set_ticks(range(1, len(BETAS) + 1), labels=["$2^{-" + f"{beta}" + "}$" for beta in range(len(BETAS))])
    ax.xaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.yaxis.set_tick_params(labelsize=FONT_SIZE)

    for patch in bplot['boxes']:
        patch.set_facecolor(TOP_COLOUR)
        patch.set_edgecolor('black')


def plot_scatter_over_beta(s: pd.Series, ax: plt.Axes, mode: str):
    if mode == "tt":
        ylabel = "avg. travel time deviation [s]"
    elif mode == "sd":
        ylabel = "avg. cum. flow deviation [vol]"
    else:
        raise ValueError("Mode must be either tt or sd")

    ax.scatter(x=range(len(BETAS)), y=s.values, color=TOP_COLOUR)

    ax.xaxis.set_ticks(range(len(BETAS)), labels=["$2^{-" + f"{beta}" + "}$" for beta in range(len(BETAS))],
                       fontsize=FONT_SIZE)
    ax.xaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.yaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.grid(True)
    ax.set_xlabel("time step size [s]", fontsize=FONT_SIZE)
    ax.set_ylabel(ylabel, fontsize=FONT_SIZE)


def plot_textbox(ax: plt.Axes, text: str, x: float, y: float, bbox_props: dict = None):
    if bbox_props is None:
        bbox_props = dict(boxstyle="round,pad=0.3", facecolor="white", edgecolor="black", alpha=0.5)
    ax.text(x, y, text, transform=ax.transAxes, fontsize=FONT_SIZE, verticalalignment='top', bbox=bbox_props)


def plot_value_count_table(ax: plt.Axes, value_counts: pd.Series, xscale: float = 0.6, yscale: float = 2.7,
                           val_col_name: str = "Value", count_col_name: str = "Count",
                           percent_col_name: str = "%") -> None:
    """
    Plot a table of value counts on the given axes.
    """
    # Create a DataFrame from the value counts
    df = pd.DataFrame({val_col_name: value_counts.index, count_col_name: value_counts.values.round()})
    df = pd.DataFrame({val_col_name: value_counts.index, count_col_name: value_counts.values.round()})
    df[percent_col_name] = (df[count_col_name] / df[count_col_name].sum() * 100).round(2)

    # Create a table and add it to the axes
    table = ax.table(cellText=df.values,
                     colLabels=df.columns,
                     cellLoc='center',
                     loc='upper left',
                     bbox=None,
                     cellColours=[["white"] * len(df.columns)] * len(df),
                     alpha=1.0)

    table.auto_set_font_size(False)
    table.set_fontsize(FONT_SIZE)
    table.scale(xscale, yscale)


def get_elementwise_difference_df(df_1: pd.DataFrame, df_2: pd.DataFrame, mode: str) -> pd.DataFrame:
    """
    Return a DataFrame with the same structure as df_1 and df_2, where numeric columns are the elementwise difference
    between df_1 and df_2 (df_1 - df_2).
    """

    if mode == "tt":
        index_col = "departure_time"
        total_col = "avg_travel_time"
    elif mode == "sd":
        index_col = "time"
        total_col = "sum_departures_total"
    else:
        raise ValueError("Mode must be either 'tt' or 'sd'")

    if index_col not in df_1.columns or index_col not in df_2.columns:
        raise ValueError(f"'{index_col}' column not found in one of the DataFrames")

    df_1 = df_1.set_index(index_col)
    df_2 = df_2.set_index(index_col)

    # Ensure both DataFrames have the same index
    if not df_1.index.equals(df_2.index):
        print(f"Index of df_1: {df_1.index}")
        print(f"Index of df_2: {df_2.index}")

        total_diff = df_1[total_col] - df_2[total_col]
        print("dfs have diff:", total_diff)
        print("which starts at", total_diff[total_diff != 0].head(10))
        print(f"where df_1 is", df_1.loc[total_diff != 0, total_col].head(10))
        print(f"where df_2 is", df_2.loc[total_diff != 0, total_col].head(10))
        raise ValueError("DataFrames do not have the same index")

    diff_df = df_1.subtract(df_2)
    diff_df = diff_df.reset_index().rename_axis(None, axis=1)
    return diff_df


def get_elementwise_avg_df(csv_path_template: str, use_random_seeds: List[int], mode: str) -> pd.DataFrame:
    """
    Read multiple summed_deps_per_time CSVs or travel_time_per_path whose paths are constructed by formatting
    csv_path_template with seed (e.g. csv_path_template.format(seed=42)), and return
    a DataFrame with the same structure where numeric columns are averaged across seeds.
    """

    if mode == "tt":
        index_col = "departure_time"
    elif mode == "sd":
        index_col = "time"
    else:
        raise ValueError("Mode must be either 'tt' or 'sd'")

    dfs: List[pd.DataFrame] = []
    for seed in use_random_seeds:
        path = csv_path_template.format(seed=seed)
        try:
            df = pd.read_csv(path)
        except FileNotFoundError as e:
            raise FileNotFoundError(f"Failed to read '{path}': {e}")
        if index_col not in df.columns:
            raise ValueError(f"'{index_col}' column not found in '{path}'")
        df = df.set_index(index_col)
        dfs.append(df)

    if not dfs:
        raise ValueError(
            f"No summed_deps_per_time CSVs successfully opened from given csv filename template '{csv_path_template}'")
    # concatenate along a new outer key (seed) so index becomes (seed, 'index_col')
    concat: pd.DataFrame = pd.concat(dfs, keys=range(len(dfs)))
    # group by time (level=1) and compute mean across seeds
    mean_df = concat.groupby(level=1).mean()
    mean_df = mean_df.reset_index().rename_axis(None, axis=1)
    # print(mean_df)
    return mean_df


def compute_deviation_to_reference_df(csv_path_template: str, betas: List, seeds: List[int], mode: str,
                                      reference_values_path: Optional[str] = None) -> pd.DataFrame:
    """
    Load CSV files for different beta and seed combinations, compute the mean absolute
    deviation from reference values across all time steps and paths.

    If reference_values_path is None, the function will compute the deviation from Nash equilibrium values.

    # TODO update text if calculation is changed.
    Currently, the absolute value is taken at the lowest level, i.e., when calculating the deviation between e.g.
    sd from the nash reference value **for a specific path, at a specific time, for a specific seed and specific beta**.
    Then the mean of those absolute deviations is taken across all paths and time steps, for that specific seed-beta
    combination.

    Returns DataFrame with shape (seeds, betas), where each cell contains the average
    absolute deviation from reference values for that seed-beta combination.
    """

    if mode == "sd":
        index_col = "time"
        # data_col = "sum_departures_avg"
        data_col = "sum_departures_path_{i}"
    elif mode == "tt":
        index_col = "departure_time"
        data_col = "avg_travel_time"
        # TODO use the below if we want absolute value deviation per path, then mean across paths and time steps
        # data_col = "avg_travel_time_path_{i}"
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

            if mode == "sd":
                # Compute deviations for all 3 paths
                deviations_over_time_per_path = []
                for path_idx in range(3):  # can be hard-coded, braess always has 3 paths
                    col_name = data_col.format(i=path_idx)

                    data_to_check = df[col_name]

                    if col_name not in df.columns:
                        raise ValueError(f"'{col_name}' not found in '{path}'")

                    if reference_values_path is None:
                        # Calculate reference values (from Nash values, between which the function is linear)
                        # Note: we don't calculate the nash values at the exact time steps that are in the csvs.
                        # Instead, we calculate them at those times + 1 time step (1/beta), because the discrete
                        # cumulative flow doesn't start at 0, but the nash flow does.
                        times_to_get_nash = (df.index + 1. / beta)
                        times_to_get_nash = times_to_get_nash[
                            0:-1]  # remove last time step, since it has no corresponding nash value

                        ref_series = get_interpolated_nash_vals(times_to_get_nash, mode, for_path=path_idx)

                        # also remove last time step from data_to_check, since it has no corresponding nash value
                        data_to_check = data_to_check[0:-1]

                        # prepend a 0 to the cumulative flow to make it start at 0 instead of 1, for better comparison with NASH lines
                        # also remove the last time step, since it has no corresponding nash value
                        # data_to_check = pd.concat([pd.Series([0]), data_to_check], ignore_index=True)[0:-1]

                    else:
                        # Read reference values from the specified path
                        reference_values = pd.read_csv(reference_values_path.format(beta=beta))
                        # Use provided reference values
                        if col_name not in reference_values.columns:
                            raise ValueError(f"Reference values must contain a column for path {col_name}")

                        ref_series = reference_values[col_name].values

                    # TODO this is where something would have to change, if the absolute value is taken at another place.
                    # Calculate absolute deviation of sd or tt w.r.t. the reference value
                    # -> the result is a vector with the deviation per time step
                    # (note: we are still in the case of per seed, per beta here)
                    deviation_over_time_at_path_i = np.abs(data_to_check - ref_series)
                    deviations_over_time_per_path.append(deviation_over_time_at_path_i)

                    # # take mean of path, only then absolute value, and then in the end we will get the avg of the abs avg deviation per path
                    # deviations_over_time_at_path_i = np.abs(np.mean(df[col_name] - nash_values))
                    # deviations_over_time_per_path.append(deviations_over_time_at_path_i)

                # Mean across all paths and time steps
                mean_deviation = np.nanmean(deviations_over_time_per_path)
            else:

                if data_col not in df.columns:
                    raise ValueError(f"'{data_col}' not found in '{path}'")

                if reference_values_path is None:
                    # Calculate reference values (from Nash values, between which the function is linear)
                    ref_series = get_interpolated_nash_vals(df.index, mode)
                else:
                    # Read reference values from the specified path
                    reference_values = pd.read_csv(reference_values_path.format(beta=beta))
                    # Use provided reference values
                    if col_name not in reference_values.columns:
                        raise ValueError(f"Reference values must contain a column for path {col_name}")

                    ref_series = reference_values[col_name].values

                deviation_over_time_of_avg_tt = np.abs(df[data_col].values - ref_series)
                mean_deviation = np.nanmean(deviation_over_time_of_avg_tt)
            result.loc[seed, beta] = mean_deviation

    # Convert to numeric type
    result = result.astype(float)
    # print(result)
    return result


def compute_deviation_to_reference_series_but_avg_first(csv_path_template: str, betas: List, seeds: List[int],
                                                        mode: str,
                                                        reference_values_path: Optional[str] = None) -> pd.Series:
    """
    #TODO docstring is copy pasted, needs update
    Load CSV files for different beta and seed combinations, compute the mean absolute
    deviation from reference values across all time steps and paths.
    If reference_values_path is None, the function will compute the deviation from Nash equilibrium values.

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
        data_col = "avg_travel_time"
        # TODO use the below if we want absolute value deviation per path, then mean across paths and time steps
        # data_col = "avg_travel_time_path_{i}"
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

        if mode == "sd":
            # Compute deviations for all 3 paths
            deviations_over_time_per_path = []
            for path_idx in range(3):  # can be hard-coded, braess always has 3 paths

                col_name = data_col.format(i=path_idx)
                data_to_check = averaged_df[col_name]

                if col_name not in averaged_df.columns:
                    raise ValueError(f"'{col_name}' not found in the averaged dataframe")

                if reference_values_path is None:
                    # Calculate reference values (from Nash values, between which the function is linear)

                    # Note: we don't calculate the nash values at the exact time steps that are in the csvs.
                    # Instead, we calculate them at those times + 1 time step (1/beta), because the discrete
                    # cumulative flow doesn't start at 0, but the nash flow does.
                    times_to_get_nash = (averaged_df.index + 1. / beta)
                    times_to_get_nash = times_to_get_nash[
                        0:-1]  # remove last time step, since it has no corresponding nash value

                    reference_series = get_interpolated_nash_vals(times_to_get_nash, mode, for_path=path_idx)

                    # also remove last time step from data_to_check, since it has no corresponding nash value
                    data_to_check = data_to_check[0:-1]
                else:
                    reference_values = pd.read_csv(reference_values_path.format(beta=beta))
                    # Use provided reference values
                    if col_name not in reference_values.columns:
                        raise ValueError(f"Reference values must contain a column for path {col_name}")
                    reference_series = (reference_values[col_name]
                                        # .reindex(averaged_df.index)
                                        .values)

                # TODO this is where something would have to change, if the absolute value is taken at another place.
                # Calculate absolute deviation of sd or tt w.r.t. the nash reference value
                # -> the result is a vector with the deviation per time step
                # (note: we are still in the case of per seed, per beta here)

                # uncomment this to first average over time (for fixed path), then take abs value, then avg over paths
                # deviation_over_time_at_path_i_avgd = np.abs(np.nanmean(averaged_df[col_name].values - nash_values))
                # deviations_over_time_per_path.append(deviation_over_time_at_path_i_avgd)

                deviation_over_time_at_path_i = np.abs(data_to_check.values - reference_series)
                deviations_over_time_per_path.append(deviation_over_time_at_path_i)

            # print(deviations_over_time_per_path)
            # Mean across all paths and time steps
            mean_deviation = np.nanmean(deviations_over_time_per_path)
        else:
            deviations_over_time_per_path = []

            col_name = data_col
            if col_name not in averaged_df.columns:
                raise ValueError(f"'{col_name}' not found in the averaged dataframe")

            if reference_values_path is None:
                # Calculate reference values (from Nash values, between which the function is linear)
                reference_series = get_interpolated_nash_vals(averaged_df.index, mode)
            else:
                reference_values = pd.read_csv(reference_values_path.format(beta=beta))
                # Use provided reference values
                if col_name not in reference_values.columns:
                    raise ValueError(f"Reference values must contain a column for path {col_name}")
                reference_series = (reference_values[col_name]
                                    # .reindex(averaged_df.index)
                                    .values)

            # TODO this is where something would have to change, if the absolute value is taken at another place.
            # Calculate absolute deviation of sd or tt w.r.t. the nash reference value
            # -> the result is a vector with the deviation per time step
            # (note: we are still in the case of per seed, per beta here)

            # uncomment this to first average over time (for fixed path), then take abs value, then avg over paths
            # deviation_over_time_at_path_i_avgd = np.abs(np.nanmean(averaged_df[col_name].values - nash_values))
            # deviations_over_time_per_path.append(deviation_over_time_at_path_i_avgd)

            deviation_over_time_at_path_i = np.abs(averaged_df[col_name].values - reference_series)
            deviations_over_time_per_path.append(deviation_over_time_at_path_i)

            # print(deviations_over_time_per_path)

            deviation_over_time_of_avg_tt = np.abs(averaged_df[col_name].values - reference_series)
            mean_deviation = np.nanmean(deviation_over_time_of_avg_tt)

        result.loc[beta] = mean_deviation

    # Convert to numeric type
    result = result.astype(float)
    # print(result)
    return result
