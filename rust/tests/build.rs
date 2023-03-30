use std::io::Write;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let destination = std::path::Path::new(&out_dir).join("tests.rs");
    let mut test_file = std::fs::File::create(&destination).unwrap();

    let mut any = false;
    for entry in std::fs::read_dir(".").unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            if let Some(name) = entry
                .file_name()
                .to_os_string()
                .into_string()
                .unwrap()
                .strip_suffix("_input.geojson")
            {
                let input_path = entry.path().display().to_string();
                let output_path = input_path.replace("_input", "_output");
                any = true;

                writeln!(test_file, "#[test]").unwrap();
                writeln!(test_file, "fn test_{name}() {{").unwrap();
                writeln!(test_file, "  test(\"{input_path}\", \"{output_path}\");").unwrap();
                writeln!(test_file, "}}").unwrap();
            }
        }
    }
    assert!(any, "Didn't find any tests");
}
