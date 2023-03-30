use std::time::Instant;

use anyhow::{bail, Result};
use geo::GeodesicLength;

use overline::{overline, Input};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        bail!("Call with input.geojson");
    }

    println!("Reading and deserializing {}", args[1]);
    let mut now = Instant::now();
    let raw = std::fs::read_to_string(&args[1])?;
    let input: Vec<Input> = geojson::de::deserialize_feature_collection_str_to_vec(&raw)?;
    println!("... took {:?}", now.elapsed());

    println!("Running overline on {} line-strings", input.len());
    now = Instant::now();
    let output = overline(&input);
    println!("... took {:?}", now.elapsed());

    // Detailed debugging
    if false {
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
    }

    println!("Writing to output.geojson");
    now = Instant::now();
    std::fs::write(
        "output.geojson",
        geojson::ser::to_feature_collection_string(&output)?,
    )?;
    println!("... took {:?}", now.elapsed());

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
