pub fn replace_placeholders_in_path_pattern(
    path_pattern: &str,
    base_output_dir: Option<&str>,
    experiment_set_name: Option<&str>,
    replanning_variant: Option<&str>,
    beta: Option<usize>,
    read_from_random: Option<usize>,
    use_random_seed: Option<usize>,
) -> String {
    let mut result = path_pattern.to_string();

    let replacements: [(&str, Option<String>); 6] = [
        ("{base_output_dir}", base_output_dir.map(str::to_string)),
        (
            "{experiment_set_name}",
            experiment_set_name.map(str::to_string),
        ),
        (
            "{replanning_variant}",
            replanning_variant.map(str::to_string),
        ),
        ("{beta}", beta.map(|v| v.to_string())),
        (
            "{read_from_random}",
            read_from_random.map(|v| v.to_string()),
        ),
        ("{use_random_seed}", use_random_seed.map(|v| v.to_string())),
    ];

    for (placeholder, value) in replacements {
        if let Some(value) = value {
            result = result.replace(placeholder, &value);
        }
    }

    result
}
