use tpe::AccountReport;

use std::{fs, path::PathBuf, process::Command};

use csv::{Reader, ReaderBuilder, Trim};

#[test]
fn example_files() {
    let input_dir = PathBuf::from("./resources/test-examples/inputs");
    let expected_dir = PathBuf::from("./resources/test-examples/expected");

    let files_to_test = fs::read_dir(input_dir.clone()).unwrap().count();

    println!("{input_dir:?}");

    for idx in 1..=files_to_test {
        let input_file = input_dir.join(format!("transactions_{idx}.csv"));
        let expected_file = expected_dir.join(format!("accounts_{idx}.csv"));

        let output = Command::new("cargo")
            .args(["run", "--", &input_file.to_str().unwrap().to_string()])
            .output()
            .unwrap();

        let output = String::from_utf8(output.stdout).unwrap();

        let mut output_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(output.as_bytes());

        let mut actual: Vec<AccountReport> = vec![];

        for record in output_reader.deserialize() {
            actual.push(record.unwrap());
        }

        let mut expected_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(expected_file)
            .unwrap();

        let mut expected: Vec<AccountReport> = vec![];

        for record in expected_reader.deserialize() {
            expected.push(record.unwrap());
        }

        actual.sort();
        expected.sort();

        assert_eq!(actual, expected);
    }
}

pub fn build_csv_reader(filepath: PathBuf) -> Reader<fs::File> {
    return ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(filepath)
        .unwrap();
}

