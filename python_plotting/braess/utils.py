from typing import Optional

import numpy as np
import pandas as pd
from matplotlib import pyplot as plt

from setup import NASH_TT_POINTS, NASH_SD_POINTS, COLORS, LABELS, FONT_SIZE, LEGEND_FONT_SIZE, \
    TOP_COLOUR


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
