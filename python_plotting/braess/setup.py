import numpy as np
import yaml

CONFIG_FILE_PATH = "experiments/config.yaml"


def get_config_file_data() -> dict:
    with open(CONFIG_FILE_PATH, "r") as f:
        config = yaml.safe_load(f)
    return config


FIG_SIZE = (12, 10)
BOXPLOT_FIG_SIZE = (12, 12)
FONT_SIZE = 45
LEGEND_FONT_SIZE = 20
TOP_COLOUR = "blue"
MID_COLOUR = "orange"
BOTTOM_COLOUR = "green"
COLORS = [TOP_COLOUR, MID_COLOUR, BOTTOM_COLOUR]
LABELS = ["top", "middle", "bottom"]

# read things from config.yaml
config_data = get_config_file_data()
ROOT_DATA_PATH = config_data.get("sim_output_base_dir")
BETAS = config_data.get("betas")
RUST_SEED_WHEN_FIXED = config_data.get("rust_seed_when_fixed")
JAVA_SEED_INDEX_WHEN_FIXED = config_data.get("java_seed_index_when_fixed")
RUST_SEEDS_TO_ITERATE_OVER = config_data.get("rust_seeds_to_iterate_over")
JAVA_SEED_INDICES_TO_ITERATE_OVER = config_data.get("java_seed_indices_to_iterate_over")

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_tt.txt"
NASH_TT_POINTS = np.array([[0, 25], [15, 55], [26.25, 66.25], [82.5, 85], [100, 85]])

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_departures.txt"
NASH_SD_POINTS = np.array([[[0, 0], [15, 0], [26.25, 0], [82.5, 18.75], [100, 27.5]],  # top
                           [[0, 0], [15, 15], [26.25, 18.75], [82.5, 37.5], [100, 37.5]],  # middle
                           [[0, 0], [15, 0], [26.25, 7.5], [82.5, 26.25], [100, 35]]])  # bottom
