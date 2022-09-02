use tpe::AccountReport;

use std::{fs, path::PathBuf, process::Command};

use csv::{ReaderBuilder, Trim};

#[test]
fn example_files() {
    let input_dir = PathBuf::from("./resources/test-examples/inputs");
    let expected_dir = PathBuf::from("./resources/test-examples/expected");

    // Running test for each file in input_dir
    let files_to_test = fs::read_dir(input_dir.clone()).unwrap().count();

    for idx in 1..=files_to_test {
        let input_file = input_dir.join(format!("transactions_{idx}.csv"));
        let expected_file = expected_dir.join(format!("accounts_{idx}.csv"));

        println!("Testing input: {input_file:?}");
        println!("Expected: {expected_file:?}");

        // Running command directly to prove everything works as expected
        let output = Command::new("cargo")
            .args(["run", "--", input_file.to_str().unwrap()])
            .output()
            .unwrap();

        println!("{}", String::from_utf8(output.stderr).unwrap());

        // Build actual from output
        let output = String::from_utf8(output.stdout).unwrap();

        let mut output_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_reader(output.as_bytes());

        let mut actual: Vec<AccountReport> = vec![];

        for record in output_reader.deserialize() {
            actual.push(record.unwrap());
        } 

        // Build expected from expected_file
        let mut expected_reader = ReaderBuilder::new()
            .trim(Trim::All)
            .from_path(expected_file)
            .unwrap();

        let mut expected: Vec<AccountReport> = vec![];

        for record in expected_reader.deserialize() {
            expected.push(record.unwrap());
        }

        // Sort to ensure order doesn't matter
        actual.sort();
        expected.sort();

        assert_eq!(actual, expected);
    }
}

