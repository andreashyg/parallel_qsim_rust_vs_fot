from typing import Optional

import numpy as np
import yaml

CONFIG_FILE_PATH = "experiments/config.yaml"


def get_config_file_data(path: str = CONFIG_FILE_PATH) -> dict:
    with open(path, "r") as f:
        config = yaml.safe_load(f)
    return config


class PlotParameters:
    FIG_SIZE = (12, 10)
    BOXPLOT_FIG_SIZE = (12, 12)
    SCATTERPLOT_FIG_SIZE = (12, 8)
    FONT_SIZE = 45
    LEGEND_FONT_SIZE = 20
    TOP_COLOUR = "blue"
    MID_COLOUR = "orange"
    BOTTOM_COLOUR = "green"
    COLORS = [TOP_COLOUR, MID_COLOUR, BOTTOM_COLOUR]
    LABELS = ["top", "middle", "bottom"]

    def __init__(self,
                 ybotlim: Optional[float | int],
                 ytoplim: Optional[float | int],
                 nash_tt_points: np.ndarray,
                 nash_sd_points: np.ndarray
                 ):
        self.ybotlim = ybotlim
        self.ytoplim = ytoplim
        self.nash_tt_points = nash_tt_points
        self.nash_sd_points = nash_sd_points

    @staticmethod
    def load_from_global_config(global_config_path: str,
                                ybotlim: Optional[float | int],
                                ytoplim: Optional[float | int]) -> PlotParameters:
        config_data = get_config_file_data(global_config_path).get("global_parameters")
        return PlotParameters(
            ybotlim=ybotlim,
            ytoplim=ytoplim,
            nash_tt_points=np.array(config_data.get("nash_tt_points")),
            nash_sd_points=np.array(config_data.get("nash_sd_points"))
        )


def load_config_data_from_global_config(global_config_path: str) -> dict:
    return get_config_file_data(
        global_config_path
    ).get("global_parameters")


def get_file_name_end_and_common_plots_patterns_from_global_config(global_config_path: str) -> tuple:
    config_data = load_config_data_from_global_config(global_config_path)
    file_name_end_pattern_with_beta_rr_ur = config_data.get("file_name_end_pattern_with_beta_rr_ur")
    file_name_end_pattern_with_beta_rr = config_data.get("file_name_end_pattern_with_beta_rr")
    file_name_end_pattern_with_beta = config_data.get("file_name_end_pattern_with_beta")
    common_plots_pattern = config_data.get("common_plots_pattern")
    return (file_name_end_pattern_with_beta_rr_ur, file_name_end_pattern_with_beta_rr,
            file_name_end_pattern_with_beta, common_plots_pattern)


def get_nash_tt_points_from_global_config(global_config_path: str) -> np.ndarray:
    config_data = load_config_data_from_global_config(global_config_path)
    nash_tt_points = np.array(config_data.get("nash_tt_points"))
    return nash_tt_points


def get_nash_sd_points_from_global_config(global_config_path: str) -> np.ndarray:
    config_data = load_config_data_from_global_config(global_config_path)
    nash_sd_points = np.array(config_data.get("nash_sd_points"))
    return nash_sd_points

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_tt.txt"
# NASH_TT_POINTS = np.array([[0, 25], [15, 55], [26.25, 66.25], [82.5, 85], [100, 85]])
#
# # from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_departures.txt"
# NASH_SD_POINTS = np.array([[[0, 0], [15, 0], [26.25, 0], [82.5, 18.75], [100, 27.5]],  # top
#                            [[0, 0], [15, 15], [26.25, 18.75], [82.5, 37.5], [100, 37.5]],  # middle
#                            [[0, 0], [15, 0], [26.25, 7.5], [82.5, 26.25], [100, 35]]])  # bottom
#
# NASH_TT_POINTS_WITH_SPILLBACK = np.array([[0, 25], [100, 225]])
# NASH_SD_POINTS_WITH_SPILLBACK = np.array([[[0, 0], [100, 0]],  # top
#                                           [[0, 0], [100, 100]],  # middle
#                                           [[0, 0], [100, 0]]])  # bottom
