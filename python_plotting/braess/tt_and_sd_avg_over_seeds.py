import sys

import matplotlib.pyplot as plt

from setup import FIG_SIZE
from utils import plot_nash_lines, plot_extracted_sd_over_time, plot_extracted_tt_over_time, get_elementwise_avg_df, \
    ExperimentSet

if __name__ == '__main__':
    # note: fixed_seed can be either a read_from_random seed or a use_random_seed, depending on seeds_to_avg_over:
    #   - if seeds_to_avg_over == "java", we fix a rust seed, i.e., fixed_seed is a use_random_seed value (for example,
    #       we take use_random_seed=1 and average over all valued of read_from_random (that is, 1..20)
    #   - if seeds_to_avg_over == "rust", we fix a java seed, i.e., fixed_seed is a read_from_random value
    # _, beta, replanning_variant, seeds_to_avg_over, fixed_seed, output_dir, read_original_java = sys.argv
    _, beta, replanning_variant, experiment_set_name, read_random, use_random, base_output_dir, output_tt_plot_path_pattern, output_sd_plot_path_pattern, input_tt_csv_path_pattern, input_sd_csv_path_pattern = sys.argv[
        0:11]
    seeds_to_use = [int(s) for s in sys.argv[11:]]

    # TODO how do I choose over what to avg?? Maybe simply via the pattern?
    #  for instance: with a input csv pattern like .../analysis/extracted_data/average_route_tts_per_deptime_beta{beta}_read_from_random_1_use_random_seed_{use_random_seed}.csv
    #  it would be clear that we want to fix read_from_random=1 and average over all use_random_seed values.
    #  QUESTION: how does the script know that beta is fixed (=1 plot per beta) while (in this case) use_random_seed should be averaged over?
    #  Maybe by a placeholder like use_random_seed_{avg_over_seeds}? And then another input that specifies the array of seeds to use.
    #  OR: the pattern stays the same. Instead, we have as input e.g. read_from_random=1 and use_random_seed=avg_over_all,
    #  and then the script knows to average over all use_random_seed values. Then a separate argument specifies the array of seeds to use for averaging (e.g. 42..61 for rust, 1..20 for java).

    if use_random.lower() == "avg_over_all":
        use_random = "{seed}"  # placeholder to be formatted later
    else:
        if read_random.lower() != "avg_over_all":
            raise ValueError("At least one of read_random or use_random must be 'avg_over_all' to average over seeds.")
        if use_random.lower() == "none":
            use_random = None
        else:
            use_random = int(use_random)

    if read_random.lower() == "avg_over_all":
        read_random = "{seed}"  # placeholder to be formatted later
    else:
        read_random = int(read_random)

    beta = int(beta)

    experiment_set = ExperimentSet(base_output_dir, replanning_variant, experiment_set_name,
                                   output_tt_plot_path_pattern,
                                   output_sd_plot_path_pattern, input_tt_csv_path_pattern, input_sd_csv_path_pattern)

    # if read_original_java.lower() == "true":
    #     read_original_java = True
    # else:
    #     read_original_java = False

    # TODO continue here: this should also use the experiment set class, so that the path to the .csv files is not hardcoded here, but rather read from the config

    # TODO: maybe, the experiment set should also contain the information about which seeds to average over, so that this is not hardcoded here, but rather read from the config

    # TODO: maybe, the experiment set should be renamed to something like "ExperimentConfig" or "ExperimentSetup", since it contains more than just the set of experiments, but also the paths to the .csv files and the seeds to average over

    # note: if use_random is None, nothing is formatted in that respect.
    # if use_random was given as "avg_over_all", we will get here a string with a placeholder {seed} that will be
    # formatted later with the actual seed values to average over.
    # same thing if read_random was given as "avg_over_all".
    tt_path = experiment_set.get_path_to_tt_csv_to_read(beta, read_random, use_random)
    sd_path = experiment_set.get_path_to_sd_csv_to_read(beta, read_random, use_random)

    # fix a use_random_seed value, but make read_from_random a placeholder to be formatted
    # if seeds_to_avg_over == "java":
    #     if read_original_java:
    #         # common for both tt and sd .csv file
    #         file_name_end = (f"_beta{beta}_"
    #                          + "read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
    #                          + f"_reformatted_original_java_data.csv"
    #                          )
    #
    #         tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/recreating_java_results/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    #         sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/recreating_java_results/analysis/extracted_data/summed_deps_per_time" + file_name_end
    #
    #     else:
    #         # common for both tt and sd .csv file
    #         file_name_end = (f"_beta{beta}_"
    #                          + "read_from_random_{seed}"  # deliberately not an f-string, used as placeholder later
    #                          + f"_use_random_seed_{fixed_seed}.csv"
    #                          )
    #         tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    #         sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end
    #
    #     seeds = JAVA_SEED_INDICES_TO_ITERATE_OVER  # read_from_random seeds for java are 1..20
    #     fixed_seed_string_for_filename = "use_random_seed"
    #
    # # fix a read_from_random value, but make use_random_seed a placeholder to be formatted
    # elif seeds_to_avg_over == "rust":
    #     if read_original_java:
    #         raise ValueError(
    #             "Parameter combination read_original_java=True and seeds_to_avg_over=rust is not supported")
    #
    #     # common for both tt and sd .csv file
    #     file_name_end = (f"_beta{beta}_read_from_random_{fixed_seed}_use_random_seed_"
    #                      + "{seed}.csv"  # deliberately not an f-string, used as placeholder later
    #                      )
    #
    #     tt_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/average_route_tts_per_deptime" + file_name_end
    #     sd_path = ROOT_DATA_PATH + f"/{replanning_variant}/varying_{seeds_to_avg_over}_seeds/analysis/extracted_data/summed_deps_per_time" + file_name_end
    #
    #     seeds = RUST_SEEDS_TO_ITERATE_OVER  # use_random seeds for rust are 42..61
    #     fixed_seed_string_for_filename = "read_from_random"
    #
    # else:
    #     raise ValueError("Seeds to average over must be either 'java' or 'rust'")

    # Note: again, if use_random is None, nothing is formatted in that respect.
    # if use_random was given as "avg_over_all", if the plot directory pattern contains a placeholder {use_random_seed}, it will be replaced by "{seed}".
    # If use_random is a specific value, it will be used as the value for the placeholder.
    # Same thing for read_random.
    # This is not necessarily expected, but normally, the plot directory pattern should not contain a placeholder for
    # the seed that is being averaged over.
    experiment_set.create_plot_dirs(beta, read_random, use_random)

    ### TT
    fig_tt, ax_tt = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_tt, mode="tt")
    tt_df = get_elementwise_avg_df(tt_path, seeds_to_use=seeds_to_use, mode="tt")
    plot_extracted_tt_over_time(ax_tt, tt_df, per_path=False)

    # try:
    #     os.makedirs(output_dir + "/tt_per_path_over_deptime")
    # except FileExistsError:
    #     pass
    #
    # if read_original_java:
    #     fig_tt.savefig(
    #         output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_original_java_data.pdf")
    # else:
    #     fig_tt.savefig(
    #         output_dir + f"/tt_per_path_over_deptime/tt_per_path_over_deptime_beta{beta}_{fixed_seed_string_for_filename}_{fixed_seed}.pdf")

    fig_tt.savefig(
        experiment_set.get_tt_plot_path(beta, read_random, use_random))

    ### SD
    fig_sd, ax_sd = plt.subplots(figsize=FIG_SIZE)
    plot_nash_lines(ax_sd, mode="sd")
    sd_df = get_elementwise_avg_df(sd_path, seeds_to_use=seeds_to_use, mode="sd")
    plot_extracted_sd_over_time(ax_sd, sd_df)

    # try:
    #     os.makedirs(output_dir + "/sd_per_path_over_time")
    # except FileExistsError:
    #     pass
    #
    # if read_original_java:
    #     fig_sd.savefig(
    #         output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_original_java_data.pdf")
    # else:
    #     fig_sd.savefig(
    #         output_dir + f"/sd_per_path_over_time/sd_per_path_over_time_beta{beta}_{fixed_seed_string_for_filename}_{fixed_seed}.pdf")

    fig_sd.savefig(
        experiment_set.get_sd_plot_path(beta, read_random, use_random))

#     TODO continue by creating a bash script for this. Then test it on non-java and java data
