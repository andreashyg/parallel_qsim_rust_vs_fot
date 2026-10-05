import matplotlib.pyplot as plt
import sys

from setup import FIG_SIZE, COMMON_PLOTS_PATTERN
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time, get_elementwise_avg_df, \
    ExperimentSet

# This is the pattern for the paths to the plots created by this script.
plot_type_specific_tt_path_pattern = "{common_plots_pattern}/avg_over_seeds/tt_per_path_over_deptime/tt_per_path_over_deptime{file_name_end}.pdf"
plot_type_specific_sd_path_pattern = "{common_plots_pattern}/avg_over_seeds/sd_per_path_over_time/sd_per_path_over_time{file_name_end}.pdf"

if __name__ == '__main__':
    _, beta, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir, input_tt_csv_path_pattern, input_sd_csv_path_pattern = sys.argv[
        0:9]
    seeds_to_use = [int(s) for s in sys.argv[9:]]

    if use_random.lower() == "avg_over_all":
        use_random_formatting_term_with_optional_placeholder = "{seed}"  # placeholder to be formatted later
    else:
        if read_random.lower() != "avg_over_all":
            raise ValueError("At least one of read_random or use_random must be 'avg_over_all' to average over seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)
        use_random_formatting_term_with_optional_placeholder = use_random

    if read_random.lower() == "avg_over_all":
        read_random_formatting_term_with_optional_placeholder = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)
        read_random_formatting_term_with_optional_placeholder = read_random

    beta = int(beta)

    # get the plot output paths by replacing the placeholder with the common plots pattern (defined in the global config)
    output_tt_plot_path_pattern = plot_type_specific_tt_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)
    output_sd_plot_path_pattern = plot_type_specific_sd_path_pattern.replace("{common_plots_pattern}",
                                                                             COMMON_PLOTS_PATTERN)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern,
                                   output_sd_plot_path_pattern, input_tt_csv_path_pattern, input_sd_csv_path_pattern)

    # note: if use_random is None, nothing is formatted in that respect.
    # if use_random was given as "avg_over_all", we will get here a string with a placeholder {seed} that will be
    # formatted later with the actual seed values to average over.
    # same thing if read_random was given as "avg_over_all".
    tt_path = experiment_set.get_path_to_tt_csv_to_read(beta, read_random_formatting_term_with_optional_placeholder,
                                                        use_random_formatting_term_with_optional_placeholder)
    sd_path = experiment_set.get_path_to_sd_csv_to_read(beta, read_random_formatting_term_with_optional_placeholder,
                                                        use_random_formatting_term_with_optional_placeholder)

    # Note: again, if use_random is None, nothing is formatted in that respect.
    # if use_random was given as "avg_over_all", if the plot directory pattern contains a placeholder {use_random_seed}, it will be replaced by "{seed}".
    # If use_random is a specific value, it will be used as the value for the placeholder.
    # Same thing for read_random.
    # This is not necessarily expected, but normally, the plot directory pattern should not contain a placeholder for
    # the seed that is being averaged over.

    # Note: here we don't use the formatting terms with placeholders, but the actual values of read_random and use_random,
    # since we don't want e.g. use_random_seed_{seed}, rather use_random_seed_avg_over_all
    experiment_set.create_plot_dirs(beta, read_random, use_random)

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_tt, mode="tt")
    tt_df = get_elementwise_avg_df(tt_path, seeds_to_use=seeds_to_use, mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, per_path=False)

    fig_tt.savefig(
        experiment_set.get_tt_plot_path(beta, read_random, use_random))

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_sd, mode="sd")
    sd_df = get_elementwise_avg_df(sd_path, seeds_to_use=seeds_to_use, mode="sd")
    plot_extracted_sd_over_time(ax_sd, sd_df)

    fig_sd.savefig(
        experiment_set.get_sd_plot_path(beta, read_random, use_random))
