import sys
from matplotlib import pyplot as plt

from setup import BOXPLOT_FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_boxplot_over_beta, compute_deviation_to_reference_df, ExperimentSet

plot_type_specific_tt_path_pattern = "{common_plots_pattern}/deviations{which_deviation}/ttAvgDeviation{which_deviation}Boxplots/ttAvgDeviation{which_deviation}Boxplot.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/deviations{which_deviation}/sdAvgDeviation{which_deviation}Boxplots/sdAvgDeviation{which_deviation}Boxplot.pdf"

if __name__ == '__main__':
    (_, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir,
     which_deviation, main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
     secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern, betas_to_use_str) = sys.argv[0:12]

    seeds_to_use = [int(s) for s in sys.argv[12:]]

    betas_to_use = [int(b) for b in betas_to_use_str.split(" ")]

    if use_random == "UseAll":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random != "UseAll":
            raise ValueError(
                "At least one of read_random or use_random must be 'UseAll' to create boxplots using all seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)

    if read_random == "UseAll":
        read_random = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra which_deviation placeholder
    output_tt_plot_path_pattern = output_tt_plot_path_pattern.replace("{which_deviation}", which_deviation)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    # also replace the extra which_deviation placeholder
    output_sd_plot_path_pattern = output_sd_plot_path_pattern.replace("{which_deviation}", which_deviation)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern, output_sd_plot_path_pattern,
                                   main_input_tt_csv_path_pattern, main_input_sd_csv_path_pattern,
                                   secondary_input_tt_csv_path_pattern, secondary_input_sd_csv_path_pattern)

    tt_path_1 = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=False)
    sd_path_1 = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=False)
    tt_path_2 = experiment_set.get_path_to_tt_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=True)
    sd_path_2 = experiment_set.get_path_to_sd_csv_to_read("{beta}",  # placeholder to be formatted later
                                                          read_random, use_random, secondary=True)

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=BOXPLOT_FIG_SIZE)

    tt_df = compute_deviation_to_reference_df(tt_path_1, betas=betas_to_use, seeds=seeds_to_use, mode="tt",
                                              reference_values_path=tt_path_2)

    plot_boxplot_over_beta(tt_df, ax_tt, "tt", betas_to_use, ytoplim=0.35)

    experiment_set.create_plot_dirs("{beta}", read_random, use_random)

    fig_tt.savefig(experiment_set.get_tt_plot_path("{beta}", read_random, use_random))

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=BOXPLOT_FIG_SIZE)

    sd_df = compute_deviation_to_reference_df(sd_path_1, betas=betas_to_use, seeds=seeds_to_use, mode="sd",
                                              reference_values_path=sd_path_2)

    plot_boxplot_over_beta(sd_df, ax_sd, "sd", betas_to_use, ytoplim=0.05)

    fig_sd.savefig(experiment_set.get_sd_plot_path("{beta}", read_random, use_random))
