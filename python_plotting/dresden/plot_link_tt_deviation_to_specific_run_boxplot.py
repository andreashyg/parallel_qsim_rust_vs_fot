import os
import sys
from typing import List

import numpy as np
import pandas as pd
from matplotlib import pyplot as plt

from python_plotting.braess.setup import PlotParameters, get_file_name_end_and_common_plots_patterns_from_global_config
from python_plotting.braess.utils import plot_boxplot_over_beta, compute_deviation_to_reference_df, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviations{which_deviation}/avgLinkTtDeviation{which_deviation}Boxplots/avgLinkTtDeviation{which_deviation}Boxplot.pdf"


# plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviations{which_deviation}/sdAvgDeviation{which_deviation}Boxplots/sdAvgDeviation{which_deviation}Boxplot.pdf"

def compute_avg_link_tt_deviation_to_reference_value(plot_params: PlotParameters, csv_path_template: str, betas: List,
                                                     seeds: List, reference_values_path: str) -> pd.DataFrame:
    # Initialize result DataFrame with seeds as index and betas as columns
    result = pd.DataFrame(index=seeds, columns=betas)

    for beta in betas:
        for seed in seeds:
            # load the csv
            path = ExperimentSet.safe_replace_placeholders(csv_path_template, seed=seed, beta=beta)
            try:
                df = pd.read_csv(path, header=None)
            except FileNotFoundError as e:
                print(f"Warning: Failed to read '{path}': {e}")
                result.loc[seed, beta] = np.nan
                continue

            reference_value = pd.read_csv(reference_values_path.format(beta=beta), header=None).values[0, 0]

            deviation_to_reference = df.values[0, 0] - reference_value
            result.loc[seed, beta] = deviation_to_reference

    result = result.astype(float)  # Ensure the DataFrame is of float type for plotting
    return result


if __name__ == '__main__':
    (_, experiment_set_name, read_random, use_random, base_output_dir,
     which_deviation, main_input_tt_csv_path_pattern,
     # main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern,
     # secondary_input_sd_csv_path_pattern,
     betas_to_use_str,
     global_config_path,
     # sd_ybotlim, sd_ytoplim,
     tt_ybotlim, tt_ytoplim) = sys.argv[0:12]

    if read_random.lower() != "none":
        raise ValueError("read_random must be 'none' for this script.")

    if use_random != "UseAll":
        raise ValueError(f"use_random must be 'UseAll' for this script, got {use_random}")

    seeds_to_use = [int(s) for s in sys.argv[16:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    # sd_ybotlim = float(sd_ybotlim)
    # sd_ytoplim = float(sd_ytoplim)
    tt_ybotlim = float(tt_ybotlim)
    tt_ytoplim = float(tt_ytoplim)

    (FILE_NAME_END_PATTERN_WITH_BETA_RR_UR,
     FILE_NAME_END_PATTERN_WITH_BETA_RR,
     FILE_NAME_END_PATTERN_WITH_BETA,
     COMMON_PLOTS_PATTERN) = get_file_name_end_and_common_plots_patterns_from_global_config(global_config_path)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra which_deviation placeholder
    output_tt_plot_path_pattern = output_tt_plot_path_pattern.replace("{which_deviation}", which_deviation)

    # output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
    #                                                                          COMMON_PLOTS_PATTERN)
    # also replace the extra which_deviation placeholder
    # output_sd_plot_path_pattern = output_sd_plot_path_pattern.replace("{which_deviation}", which_deviation)

    experiment_set = ExperimentSet(global_config_path, base_output_dir, experiment_set_name,
                                   output_tt_plot_path_pattern, "",  # output_sd_plot_path_pattern,
                                   main_input_tt_csv_path_pattern, "",
                                   secondary_input_tt_csv_path_pattern, "")

    tt_path_1 = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          "", "{seed}", secondary=False)
    # sd_path_1 = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
    #                                                       read_random, use_random, secondary=False)
    tt_path_2 = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          "", "{seed}", secondary=True)
    # sd_path_2 = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
    #                                                       read_random, use_random, secondary=True)

    ### TT
    tt_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, ybotlim=tt_ybotlim,
                                                                ytoplim=tt_ytoplim)  # 0.35)
    fig_tt, ax_tt = plt.subplots(figsize=tt_plot_parameters.BOXPLOT_FIG_SIZE)

    tt_df = compute_avg_link_tt_deviation_to_reference_value(tt_plot_parameters, tt_path_1, betas=betas_to_use,
                                                             seeds=seeds_to_use,
                                                             reference_values_path=tt_path_2)

    plot_boxplot_over_beta(tt_df, ax_tt, "avg_link_tt", betas_to_use, plot_params=tt_plot_parameters)

    tt_plot_dir = os.path.dirname(experiment_set.get_tt_plot_path("", "", ""))
    os.makedirs(tt_plot_dir, exist_ok=True)
    fig_tt.savefig(experiment_set.get_tt_plot_path("", "", ""),
                   bbox_inches="tight")

    # ### SD
    # sd_plot_parameters = PlotParameters.load_from_global_config(experiment_set.global_config_path, ybotlim=sd_ybotlim,
    #                                                             ytoplim=sd_ytoplim)  # 0.05)
    # fig_sd, ax_sd = plt.subplots(figsize=sd_plot_parameters.BOXPLOT_FIG_SIZE)
    #
    # sd_df = compute_deviation_to_reference_df(sd_plot_parameters, sd_path_1, betas=betas_to_use, seeds=seeds_to_use,
    #                                           mode="sd",
    #                                           reference_values_path=sd_path_2)
    #
    # plot_boxplot_over_beta(sd_df, ax_sd, "sd", betas_to_use, plot_params=sd_plot_parameters)
    #
    # fig_sd.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random),
    #                bbox_inches="tight")
