import numpy as np
import yaml

CONFIG_FILE_PATH = "experiments/config.yaml"


def get_config_file_data(path: str = CONFIG_FILE_PATH) -> dict:
    with open(path, "r") as f:
        config = yaml.safe_load(f)
    return config


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

# read things from global_config.yaml
config_data_from_global_config = get_config_file_data(
    "experiments/compare_braess_to_java/experiment_sets/global_config.yaml").get("global_parameters")
FILE_NAME_END_PATTERN_WITH_BETA_RR_UR = config_data_from_global_config.get("file_name_end_pattern_with_beta_rr_ur")
FILE_NAME_END_PATTERN_WITH_BETA_RR = config_data_from_global_config.get("file_name_end_pattern_with_beta_rr")
FILE_NAME_END_PATTERN_WITH_BETA = config_data_from_global_config.get("file_name_end_pattern_with_beta")
COMMON_PLOTS_PATTERN = config_data_from_global_config.get("common_plots_pattern")

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_tt.txt"
NASH_TT_POINTS = np.array([[0, 25], [15, 55], [26.25, 66.25], [82.5, 85], [100, 85]])

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_departures.txt"
NASH_SD_POINTS = np.array([[[0, 0], [15, 0], [26.25, 0], [82.5, 18.75], [100, 27.5]],  # top
                           [[0, 0], [15, 15], [26.25, 18.75], [82.5, 37.5], [100, 37.5]],  # middle
                           [[0, 0], [15, 0], [26.25, 7.5], [82.5, 26.25], [100, 35]]])  # bottom
