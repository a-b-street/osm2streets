use std::io::Write;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("tests.rs");
    let mut test_file = std::fs::File::create(&destination).unwrap();

    let mut any = false;
    for entry in std::fs::read_dir("src").unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let name = entry.file_name().to_os_string().into_string().unwrap();
        let path = entry.path().display().to_string();
        any = true;

        writeln!(test_file, "#[test]").unwrap();
        writeln!(test_file, "fn test_{name}() {{").unwrap();
        writeln!(test_file, "  test(\"{path}\").unwrap();").unwrap();
        writeln!(test_file, "}}").unwrap();
    }
    assert!(any, "Didn't find any tests");
}
