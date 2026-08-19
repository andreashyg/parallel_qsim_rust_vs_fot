use polars::prelude::*;
use std::error::Error;
use std::fs::{File, create_dir_all};
use std::path::Path;

/// Reads a CSV file, renames all columns, and writes the updated CSV.
///
/// Panics if the CSV has no header names or if the number of provided names does not match
/// the number of columns.
pub fn rename_csv_columns(
    input_file: &Path,
    output_file: &Path,
    new_column_names: &[String],
) -> Result<(), Box<dyn Error>> {
    let mut df = CsvReadOptions::default()
        .with_parse_options(CsvParseOptions::default().with_separator(b'\t'))
        .try_into_reader_with_file_path(Some(input_file.to_path_buf()))?
        .finish()?;

    let original_column_names = df
        .get_column_names()
        .iter()
        .map(|name| name.as_str().to_string())
        .collect::<Vec<_>>();

    assert!(
        !original_column_names.is_empty()
            && original_column_names
                .iter()
                .all(|name| !name.trim().is_empty()),
        "Input CSV has no valid column names."
    );

    assert_eq!(
        original_column_names.len(),
        new_column_names.len(),
        "Number of new column names ({}) does not match number of CSV columns ({}).",
        new_column_names.len(),
        original_column_names.len()
    );

    for (old_name, new_name) in original_column_names.iter().zip(new_column_names.iter()) {
        df.rename(old_name, new_name.clone().into())?;
    }

    create_dir_all(
        output_file
            .parent()
            .expect("Output file has no parent directory"),
    )?;

    let mut file = File::create(output_file)?;
    CsvWriter::new(&mut file).finish(&mut df)?;

    Ok(())
}
