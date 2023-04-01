#[cfg(test)]
mod tests {
    use overline::{overline, Input, Output};

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    fn test(input_path: &str, output_path: &str) {
        let input: Vec<Input> = geojson::de::deserialize_feature_collection_str_to_vec(
            &std::fs::read_to_string(input_path).unwrap(),
        )
        .unwrap();
        let actual_output = overline(&input);
        let expected_output: Vec<Output> = geojson::de::deserialize_feature_collection_str_to_vec(
            &std::fs::read_to_string(output_path).unwrap(),
        )
        .unwrap();
        assert_eq!(actual_output, expected_output);
    }
}
