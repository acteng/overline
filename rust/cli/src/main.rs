use std::time::Instant;

use anyhow::{bail, Result};
use geo::GeodesicLength;
use geojson::{Feature, FeatureCollection, GeoJson};

use overline::{feature_to_line_string, overline, Output};

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

    // TODO Make a CLI
    if args[1] == "../route_segments_minimal.geojson" {
        aggregate_and_write(
            input,
            output,
            vec![
                ("All".into(), Aggregation::SumFloat),
                ("Driving.a.car.or.van".into(), Aggregation::SumFloat),
                ("Name".into(), Aggregation::KeepAny),
            ],
        )?;
    } else if args[1] == "cycle_routes_london.geojson" {
        aggregate_and_write(
            input,
            output,
            vec![
                ("all".into(), Aggregation::SumFloat),
                ("bicycle".into(), Aggregation::SumFloat),
                ("foot".into(), Aggregation::SumFloat),
            ],
        )?;
    } else if args[1] == "tests/atip_input.geojson" {
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

        println!("Writing to output.geojson");
        now = Instant::now();
        std::fs::write(
            "output.geojson",
            geojson::ser::to_feature_collection_string(&output)?,
        )?;
        println!("... took {:?}", now.elapsed());
    }

    Ok(())
}

fn aggregate_and_write(
    input: Vec<Feature>,
    output: Vec<Output>,
    properties: Vec<(String, Aggregation)>,
) -> Result<()> {
    println!(
        "Aggregating properties on {} grouped line-strings",
        output.len()
    );
    let mut now = Instant::now();
    let final_output = aggregate_properties(&input, &output, properties);
    println!("... took {:?}", now.elapsed());

    println!("Writing to output.geojson");
    now = Instant::now();
    std::fs::write(
        "output.geojson",
        GeoJson::from(FeatureCollection {
            bbox: None,
            features: final_output,
            foreign_members: None,
        })
        .to_string(),
    )?;
    println!("... took {:?}", now.elapsed());
    Ok(())
}

enum Aggregation {
    /// Copy the value of this property from any input feature containing it. If the property
    /// differs among the input, it's undefined which value will be used.
    KeepAny,
    /// Sum this property as a floating point.
    SumFloat,
}

fn aggregate_properties(
    input: &Vec<Feature>,
    grouped_indices: &Vec<Output>,
    properties: Vec<(String, Aggregation)>,
) -> Vec<Feature> {
    let mut output = Vec::new();
    for grouped in grouped_indices {
        // Copy the geometry
        let mut feature = Feature {
            geometry: Some(geojson::Geometry {
                value: geojson::Value::from(&grouped.geometry),
                bbox: None,
                foreign_members: None,
            }),
            properties: None,
            bbox: None,
            id: None,
            foreign_members: None,
        };
        // Aggregate each specified property
        for (key, aggregation) in &properties {
            // Ignore features without this property
            let mut values = grouped
                .indices
                .iter()
                .flat_map(|i| input[*i].property(&key));
            match aggregation {
                Aggregation::KeepAny => {
                    if let Some(value) = values.next() {
                        feature.set_property(key, value.clone());
                    }
                }
                Aggregation::SumFloat => {
                    feature.set_property(key, values.flat_map(|x| x.as_f64()).sum::<f64>());
                }
            }
        }
        output.push(feature);
    }
    output
}

/* Test cases:
 * same line
 * same line, reversed
 * cross at a single point
 * slightly offset coordinates
 *
 * TODO Include a little Leaflet viewer that can load an input/output file pair and display it
 */
