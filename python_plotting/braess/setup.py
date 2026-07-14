import numpy as np

FIG_SIZE = (12, 8)
FONT_SIZE = 28
LEGEND_FONT_SIZE = 20
TOP_COLOUR = "blue"
MID_COLOUR = "orange"
BOTTOM_COLOUR = "green"
COLORS = [TOP_COLOUR, MID_COLOUR, BOTTOM_COLOUR]
LABELS = ["top", "middle", "bottom"]
ROOT_DATA_PATH = "../runs-svn/Abschlussarbeiten/2026/andreas-hygrell-rust-vs-fot/compare_braess_to_java/"

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_tt.txt"
NASH_TT_POINTS = np.array([[0, 25], [15, 55], [26.25, 66.25], [82.5, 85], [100, 85]])

# from "../../../runs-svn/braess/refinement/no_spillback_scenario/braess_nash_departures.txt"
NASH_SD_POINTS = np.array([[[0, 0], [15, 0], [26.25, 0], [82.5, 18.75], [100, 27.5]],  # top
                           [[0, 0], [15, 15], [26.25, 18.75], [82.5, 37.5], [100, 37.5]],  # middle
                           [[0, 0], [15, 0], [26.25, 7.5], [82.5, 26.25], [100, 35]]])  # bottom

BETAS = [1, 2, 4, 8, 16]
