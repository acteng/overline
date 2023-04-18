#[cfg(test)]
mod tests {
    use geojson::FeatureCollection;
    use overline::{aggregate_properties, overline, Aggregation};

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    fn test(input_path: &str, output_path: &str) {
        // Just compare as strings. Upon failure, we write and ask the user to check those anyway.
        let input_string = std::fs::read_to_string(input_path).unwrap();

        let input = input_string.parse::<FeatureCollection>().unwrap().features;
        let grouped_indices = overline(&input);
        let actual_output = FeatureCollection {
            features: aggregate_properties(
                &input,
                &grouped_indices,
                vec![("foot".to_string(), Aggregation::Sum)],
            ),
            bbox: None,
            foreign_members: None,
        }
        .to_string();

        let expected_output = std::fs::read_to_string(output_path).unwrap();
        if actual_output != expected_output {
            let actual_path = format!("{output_path}_ACTUAL");
            std::fs::write(&actual_path, actual_output).unwrap();
            panic!("Failed for {input_path}. Compare actual output {actual_path} with expected {output_path}");
        }
    }
}
