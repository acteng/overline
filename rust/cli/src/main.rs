use anyhow::Result;
use geo::GeodesicLength;

use overline::{overline, Input};

fn main() -> Result<()> {
    let raw = std::fs::read_to_string("tests/atip_input.geojson")?;
    let input: Vec<Input> = geojson::de::deserialize_feature_collection_str_to_vec(&raw)?;
    let output = overline(&input);
    println!("Input:");
    for (idx, line) in input.iter().enumerate() {
        println!(
            "- {idx} has foot={}, length={}",
            line.foot,
            line.geometry.geodesic_length()
        );
    }
    println!("Output:");
    for line in &output {
        let sum_feet: usize = line.indices.iter().map(|i| input[*i].foot).sum();
        println!(
            "- length={}, indices {:?}, sum of feet {}",
            line.geometry.geodesic_length(),
            line.indices,
            sum_feet
        );
    }
    Ok(())
}

/* Test cases:
 * same line
 * same line, reversed
 * cross at a single point
 * slightly offset coordinates
 *
 * TODO Include a little Leaflet viewer that can load an input/output file pair and display it
 */
