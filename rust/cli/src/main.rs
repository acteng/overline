use std::time::Instant;

use anyhow::{bail, Result};
use geo::GeodesicLength;
use geojson::{Feature, GeoJson};

use overline::{feature_to_line_string, overline};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        bail!("Call with input.geojson");
    }

    println!("Reading and deserializing {}", args[1]);
    let mut now = Instant::now();
    let geojson: GeoJson = std::fs::read_to_string(&args[1])?.parse()?;
    let input: Vec<Feature> = if let GeoJson::FeatureCollection(collection) = geojson {
        collection.features
    } else {
        bail!("Input isn't a FeatureCollection");
    };
    println!("... took {:?}", now.elapsed());

    println!("Running overline on {} line-strings", input.len());
    now = Instant::now();
    let output = overline(&input);
    println!("... took {:?}", now.elapsed());

    // Detailed debugging
    if true {
        fn foot(f: &Feature) -> f64 {
            f.property("foot").unwrap().as_f64().unwrap()
        }

        println!("Input:");
        for (idx, line) in input.iter().enumerate() {
            if let Some(geom) = feature_to_line_string(line) {
                println!(
                    "- {idx} has foot={}, length={}",
                    foot(line),
                    geom.geodesic_length()
                );
            }
        }
        println!("Output:");
        for line in &output {
            let sum_feet: f64 = line.indices.iter().map(|i| foot(&input[*i])).sum();
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
