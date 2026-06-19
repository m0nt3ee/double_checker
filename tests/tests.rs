#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use predicate::str::contains;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_missing_argument() {
        let mut cmd = Command::cargo_bin("dbcheck").unwrap();

        cmd.assert()
            .failure()
            .stderr(contains("the following required arguments were not provided"));
    }

    #[test]
    fn test_file_not_found() {
        let mut cmd = Command::cargo_bin("dbcheck").unwrap();
        cmd.arg("random_file.txt");

        cmd.assert()
            .failure()
            .stderr(contains("Error"));
    }

    #[test]
    fn test_correct_usage() {
        let temp_file_path = "test_dummy.txt";
        let mut file = File::create(temp_file_path).unwrap();
        writeln!(file, "testing").unwrap();

        let mut cmd = Command::cargo_bin("dbcheck").unwrap();
        cmd.arg(temp_file_path);

        cmd.assert()
            .success()
            .stdout(contains("Found"));

        std::fs::remove_file(temp_file_path).unwrap();
    }
}