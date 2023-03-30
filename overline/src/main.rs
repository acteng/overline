use std::collections::HashSet;

use anyhow::Result;
use geo::GeodesicLength;
use ordered_float::NotNan;
use serde::Deserialize;

#[derive(Deserialize)]
struct Input {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: geo::LineString<f64>,
    foot: usize,
}

struct Output {
    geometry: geo::LineString<f64>,
    // The indices of Input lines that matched to this segment
    inputs: HashSet<usize>,
}

fn main() -> Result<()> {
    let raw = std::fs::read_to_string("input.geojson")?;
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
        println!(
            "- length={}, indices {:?}",
            line.geometry.geodesic_length(),
            line.inputs
        );
    }
    Ok(())
}

fn overline(input: &Vec<Input>) -> Vec<Output> {
    // Split lines by these
    let mut vertices: HashSet<HashedPoint> = HashSet::new();

    // Vertices come from endpoints, but also any intermediate points shared by two lines
    // TODO Crossing lines?
    let mut seen: HashSet<HashedPoint> = HashSet::new();

    for line in input {
        let mut iterator = line.geometry.points().map(HashedPoint::new).peekable();
        let mut first = true;
        while let Some(pt) = iterator.next() {
            if first || iterator.peek().is_none() || seen.contains(&pt) {
                vertices.insert(pt);
            } else {
                seen.insert(pt);
            }
            first = false;
        }
    }

    let mut output = Vec::new();
    for (idx, line) in input.iter().enumerate() {
        let mut pts = Vec::new();
        for pt in line.geometry.points() {
            pts.push(pt);
            if vertices.contains(&HashedPoint::new(pt)) && pts.len() > 1 {
                output.push(Output {
                    geometry: std::mem::take(&mut pts).into(),
                    inputs: HashSet::from([idx]),
                });
            }
        }
    }
    // TODO Be careful, if we index lines by (start, end), because there could be multiple lines
    // between the same pair

    // Merge identical outputs

    output
}

// Assume there are no precision issues with the input. If we wanted to quantize, we'd do it here.
#[derive(PartialEq, Eq, Hash, Debug)]
struct HashedPoint {
    x: NotNan<f64>,
    y: NotNan<f64>,
}
impl HashedPoint {
    fn new(pt: geo::Point<f64>) -> Self {
        Self {
            x: NotNan::new(pt.x()).unwrap(),
            y: NotNan::new(pt.y()).unwrap(),
        }
    }
}

/* Test cases:
 * same line
 * same line, reversed
 * cross at a single point
 * slightly offset coordinates
 */
