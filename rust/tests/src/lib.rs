#[cfg(test)]
mod tests {
    use geojson::FeatureCollection;
    use overline::{aggregate_properties, overline, Aggregation, Options};

    #[test]
    fn test_atip() {
        test(
            "atip_input.geojson",
            "atip_output.geojson",
            Options::default(),
        );
    }

    #[test]
    fn test_direction() {
        //test("direction_input.geojson", "direction_input.geojson");
        test(
            "direction_input.geojson",
            "direction_output.geojson",
            Options {
                ignore_direction: true,
            },
        );
    }

    fn test(input_path: &str, output_path: &str, options: Options) {
        // Just compare as strings. Upon failure, we write and ask the user to check those anyway.
        let input_string = std::fs::read_to_string(input_path).unwrap();

        let input = input_string.parse::<FeatureCollection>().unwrap().features;
        let grouped_indices = overline(&input, options);
        let actual_output = serde_json::to_string_pretty(&FeatureCollection {
            features: aggregate_properties(
                &input,
                &grouped_indices,
                vec![("foot".to_string(), Aggregation::Sum)],
            ),
            bbox: None,
            foreign_members: None,
        })
        .unwrap();

        let expected_output = std::fs::read_to_string(output_path);
        if expected_output
            .map(|expected| actual_output != expected)
            .unwrap_or(true)
        {
            let actual_path = format!("{output_path}_ACTUAL");
            std::fs::write(&actual_path, actual_output).unwrap();
            panic!("Failed for {input_path}. Compare actual output {actual_path} with expected {output_path}");
        }
    }

    // Manually use this to format a GeoJSON file, so that string comparison (and manual diffing)
    // is easy
    #[allow(unused)]
    fn format_input(path: &str) {
        let fc = std::fs::read_to_string(path)
            .unwrap()
            .parse::<FeatureCollection>()
            .unwrap();
        std::fs::write(path, serde_json::to_string_pretty(&fc).unwrap()).unwrap();
    }
}
