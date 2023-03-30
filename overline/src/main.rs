use std::collections::HashMap;

use anyhow::Result;
use geo::GeodesicLength;
use ordered_float::NotNan;
use serde::Deserialize;

// TODO Never aggregate across OSM ID threshold. Plumb through an optional property to restrict
// aggregation.

#[derive(Deserialize)]
struct Input {
    #[serde(deserialize_with = "geojson::de::deserialize_geometry")]
    geometry: geo::LineString<f64>,
    foot: usize,
}

struct Output {
    geometry: geo::LineString<f64>,
    // The indices of Input lines that matched to this segment
    indices: Vec<usize>,
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

fn overline(input: &Vec<Input>) -> Vec<Output> {
    // For every individual (directed) line segment, record the index of inputs matching there
    let mut line_segments: HashMap<(HashedPoint, HashedPoint), Vec<usize>> = HashMap::new();
    for (idx, input) in input.iter().enumerate() {
        for line in input.geometry.lines().map(HashedPoint::new_line) {
            line_segments.entry(line).or_insert_with(Vec::new).push(idx);
        }
    }
    // TODO We also need to split some line segments if they're not matching up at existing points.

    // Then look at each input, accumulating points as long all the indices match
    let mut output = Vec::new();
    for (idx, input) in input.iter().enumerate() {
        // This state is reset as we look through this input's points
        let mut pts_so_far = Vec::new();
        let mut indices_so_far = Vec::new();
        let mut keep_this_output = false;

        for line in input.geometry.lines() {
            // The segment is guaranteed to exist
            let indices = &line_segments[&HashedPoint::new_line(line)];
            if pts_so_far.is_empty() {
                pts_so_far.push(line.start);
                pts_so_far.push(line.end);
                indices_so_far = indices.clone();
                // Say we're processing input 2, and we have a segment with indices [2, 5]. We want
                // to add it to output. But later we'll work on input 5 and see the same segment
                // with indices [2, 5]. We don't want to add it again, so we'll skip it using the
                // logic below, since we process input in order.
                keep_this_output = indices_so_far.iter().all(|i| *i >= idx);
            } else if &indices_so_far == indices {
                assert_eq!(*pts_so_far.last().unwrap(), line.start);
                pts_so_far.push(line.end);
            } else {
                // The overlap ends here
                let add = Output {
                    geometry: std::mem::take(&mut pts_so_far).into(),
                    indices: std::mem::take(&mut indices_so_far),
                };
                if keep_this_output {
                    output.push(add);
                }
                keep_this_output = false;
            }
        }
        // This input ended; add to output if needed
        if !pts_so_far.is_empty() && keep_this_output {
            output.push(Output {
                geometry: pts_so_far.into(),
                indices: indices_so_far,
            });
        }
    }

    output
}

// Assume there are no precision issues with the input. If we wanted to quantize, we'd do it here.
#[derive(PartialEq, Eq, Hash, Debug)]
struct HashedPoint {
    x: NotNan<f64>,
    y: NotNan<f64>,
}
impl HashedPoint {
    fn new(coord: geo::Coord<f64>) -> Self {
        Self {
            x: NotNan::new(coord.x).unwrap(),
            y: NotNan::new(coord.y).unwrap(),
        }
    }

    fn new_line(line: geo::Line<f64>) -> (Self, Self) {
        (Self::new(line.start), Self::new(line.end))
    }
}

/* Test cases:
 * same line
 * same line, reversed
 * cross at a single point
 * slightly offset coordinates
 *
 * TODO Include a little Leaflet viewer that can load an input/output file pair and display it
 */
