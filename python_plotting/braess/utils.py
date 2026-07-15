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


def plot_extracted_sd_over_time(ax: plt.Axes, sd_df: pd.DataFrame):
    for i in range(3):
        sd_df.plot(x="time", y=f"sum_departures_path_{i}", kind="scatter", ax=ax, label=LABELS[i], color=COLORS[i])

    ax.set_xlabel("time [s]", fontsize=FONT_SIZE)
    ax.set_ylabel("cumulative flow [vol]", fontsize=FONT_SIZE)
    ax.xaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.yaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.legend(fontsize=LEGEND_FONT_SIZE)


def plot_extracted_tt_over_time(ax: plt.Axes, tt_df: pd.DataFrame, per_path: bool = True):
    if per_path:
        for i in range(3):
            tt_df.plot(x="departure_time", y=f"avg_travel_time_path_{i}", kind="scatter", ax=ax, label=LABELS[i],
                       color=COLORS[i])
    else:
        tt_df.plot(x="departure_time", y="avg_travel_time", kind="scatter", ax=ax, label="avg. path", color=TOP_COLOUR)

    ax.set_xlabel("departure time [s]", fontsize=FONT_SIZE)
    ax.set_ylabel("travel time [s]", fontsize=FONT_SIZE)
    ax.xaxis.set_tick_params(labelsize=FONT_SIZE)
    ax.yaxis.set_tick_params(labelsize=FONT_SIZE)
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
    ax.set_xlabel("time step size [s]", fontsize=FONT_SIZE)
    ax.set_ylabel(ylabel, fontsize=FONT_SIZE)


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
    return mean_df
