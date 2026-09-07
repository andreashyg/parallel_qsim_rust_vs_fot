import pandas as pd


def verify_csv_files_identical(file1, file2):
    """
    Verify that two CSV files are identical by comparing their contents.

    Parameters:
    file1 (str): Path to the first CSV file.
    file2 (str): Path to the second CSV file.

    Returns:
    bool: True if the files are identical, False otherwise.
    """
    try:
        # Read both CSV files into DataFrames
        df1 = pd.read_csv(file1)
        df2 = pd.read_csv(file2)

        # Compare the DataFrames
        if df1.equals(df2):
            print(f"The files '{file1}' and '{file2}' are identical.")
            return True
        else:
            print(f"The files '{file1}' and '{file2}' are NOT identical.")
            print("Details of differences:")
            # Find differences
            diff = df1.compare(df2)
            print(diff)
            return False

    except Exception as e:
        print(f"An error occurred while comparing the files: {e}")
        return False


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 3:
        print("Usage: python verify_csv_files_identical.py <file1.csv> <file2.csv>")
        sys.exit(1)

    file1 = sys.argv[1]
    file2 = sys.argv[2]

    verify_csv_files_identical(file1, file2)
