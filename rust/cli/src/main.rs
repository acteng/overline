use std::time::Instant;

use anyhow::{bail, Result};
use clap::{Arg, ArgAction, Command};
use geo::GeodesicLength;
use geojson::{Feature, FeatureCollection, GeoJson};
use ordered_float::NotNan;
use rayon::prelude::*;

use overline::{feature_to_line_string, overline, Output};

fn main() -> Result<()> {
    let mut args = Command::new("overline")
        .author("Dustin Carlino, dabreegster@gmail.com")
        // TODO Version, about
        .arg(Arg::new("FILE").help("GeoJSON input with LineStrings").required(true))
        .arg(Arg::new("output").short('o').long("output").help("Write GeoJSON output here").default_value("output.geojson"))
        // TODO Just indices, or aggregate something?
        .arg(Arg::new("summary").short('s').long("summary").help("Print a summary of the input and output, summing the one specified numeric property").action(ArgAction::Set))
        .arg(Arg::new("keep_any").long("keep_any").action(ArgAction::Append).help("Copy the value of this property from any input feature containing it."))
        .arg(Arg::new("sum").long("sum").action(ArgAction::Append).help("Sum this property as a floating point."))
        .arg(Arg::new("min").long("min").action(ArgAction::Append).help("Minimum of this property as a floating point."))
        .arg(Arg::new("max").long("max").action(ArgAction::Append).help("Maximum of this property as a floating point."))
        .arg(Arg::new("mean").long("mean").action(ArgAction::Append).help("Mean (average) of this property as a floating point."))
        .get_matches();
    let input_path = args.remove_one::<String>("FILE").unwrap();
    let output_path = args.remove_one::<String>("output").unwrap();

    println!("Reading and deserializing {input_path}");
    let mut now = Instant::now();
    let geojson: GeoJson = std::fs::read_to_string(input_path)?.parse()?;
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

    if let Some(sum_property) = args.get_one::<String>("summary") {
        let get_property = |f: &Feature| {
            f.property(&sum_property)
                .expect(&format!("don't have property {sum_property}"))
                .as_f64()
                .expect(&format!("property {sum_property} isn't numeric"))
        };

        println!("Input:");
        for (idx, line) in input.iter().enumerate() {
            if let Some(geom) = feature_to_line_string(line) {
                println!(
                    "- {idx} has {sum_property}={}, length={}",
                    get_property(line),
                    geom.geodesic_length()
                );
            }
        }
        println!("Output:");
        for line in &output {
            let sum: f64 = line.indices.iter().map(|i| get_property(&input[*i])).sum();
            println!(
                "- length={}, indices {:?}, sum of {sum_property} {}",
                line.geometry.geodesic_length(),
                line.indices,
                sum
            );
        }
    }

    let mut aggregate_props = Vec::new();
    for (name, x) in [
        ("keep_any", Aggregation::KeepAny),
        ("sum", Aggregation::Sum),
        ("min", Aggregation::Min),
        ("max", Aggregation::Max),
        ("mean", Aggregation::Mean),
    ] {
        if let Some(values) = args.remove_many::<String>(name) {
            for key in values {
                aggregate_props.push((key, x));
            }
        }
    }

    if aggregate_props.is_empty() {
        println!("Writing with indices to {output_path}");
        now = Instant::now();
        std::fs::write(
            output_path,
            geojson::ser::to_feature_collection_string(&output)?,
        )?;
        println!("... took {:?}", now.elapsed());
    } else {
        println!(
            "Aggregating properties on {} grouped line-strings",
            output.len()
        );
        let mut now = Instant::now();
        let final_output = aggregate_properties(&input, &output, aggregate_props);
        println!("... took {:?}", now.elapsed());

        println!("Writing to {output_path}");
        now = Instant::now();
        std::fs::write(
            output_path,
            GeoJson::from(FeatureCollection {
                bbox: None,
                features: final_output,
                foreign_members: None,
            })
            .to_string(),
        )?;
        println!("... took {:?}", now.elapsed());
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum Aggregation {
    /// Copy the value of this property from any input feature containing it. If the property
    /// differs among the input, it's undefined which value will be used.
    KeepAny,
    /// Sum this property as a floating point.
    Sum,
    /// Minimum of this property as a floating point.
    Min,
    /// Maximum of this property as a floating point.
    Max,
    /// Mean (average) of this property as a floating point.
    Mean,
}
// TODO Percentile
// TODO The value coming from the longest piece of LineString

fn aggregate_properties(
    input: &Vec<Feature>,
    grouped_indices: &Vec<Output>,
    properties: Vec<(String, Aggregation)>,
) -> Vec<Feature> {
    grouped_indices
        .par_iter()
        .map(|grouped| {
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
                    Aggregation::Sum => {
                        feature.set_property(key, values.flat_map(|x| x.as_f64()).sum::<f64>());
                    }
                    Aggregation::Min => {
                        if let Some(min) =
                            values.flat_map(|x| x.as_f64()).flat_map(NotNan::new).min()
                        {
                            feature.set_property(key, min.into_inner());
                        }
                    }
                    Aggregation::Max => {
                        if let Some(max) =
                            values.flat_map(|x| x.as_f64()).flat_map(NotNan::new).max()
                        {
                            feature.set_property(key, max.into_inner());
                        }
                    }
                    Aggregation::Mean => {
                        let mut sum = 0.0;
                        let mut count = 0;
                        for x in values.flat_map(|x| x.as_f64()) {
                            sum += x;
                            count += 1;
                        }
                        if count > 0 {
                            feature.set_property(key, sum / count as f64);
                        }
                    }
                }
            }
            feature
        })
        .collect()
}

/* Test cases:
 * same line
 * same line, reversed
 * cross at a single point
 * slightly offset coordinates
 *
 * TODO Include a little Leaflet viewer that can load an input/output file pair and display it
 */
