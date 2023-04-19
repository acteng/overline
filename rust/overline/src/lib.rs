use std::collections::HashMap;

use geo::{HaversineBearing, Winding};
use geojson::Feature;
use ordered_float::NotNan;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub use aggregation::{aggregate_properties, Aggregation};

mod aggregation;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    #[serde(
        serialize_with = "geojson::ser::serialize_geometry",
        deserialize_with = "geojson::de::deserialize_geometry"
    )]
    pub geometry: geo::LineString<f64>,
    /// The indices of Input lines that matched to this segment
    pub indices: Vec<usize>,
}

#[derive(Default)]
pub struct Options {
    /// TODO describe. may reverse input geometry.
    pub ignore_direction: bool,
}

/// Ignores anything aside from LineStrings. Returns LineStrings chopped up to remove overlap, with
/// exactly one property -- `indices`, an array of numbers indexing the input that share that
/// geometry.
pub fn overline(input: &Vec<Feature>, options: Options) -> Vec<Output> {
    // Extract LineStrings from input
    let input_linestrings: Vec<Option<geo::LineString<f64>>> = input
        .par_iter()
        .map(|f| {
            feature_to_line_string(f).map(|mut linestring| {
                if options.ignore_direction {
                    // TODO Do we need to project to euclidean first?
                    println!("input is cw? {:?}", linestring.winding_order());
                    linestring.make_cw_winding();

                    let pt1 = linestring.0[0];
                    let pt2 = *linestring.0.last().unwrap();
                    let bearing = geo::Point::from(pt1).haversine_bearing(geo::Point::from(pt2));
                    if bearing < 0.0 {
                        linestring.0.reverse();
                    }
                }
                linestring
            })
        })
        .collect();

    // For every individual (directed) line segment, record the index of inputs matching there
    let mut line_segments: HashMap<(HashedPoint, HashedPoint), Vec<usize>> = HashMap::new();
    for (idx, maybe_linestring) in input_linestrings.iter().enumerate() {
        if let Some(ref linestring) = maybe_linestring {
            for line in linestring.lines().map(HashedPoint::new_line) {
                line_segments.entry(line).or_insert_with(Vec::new).push(idx);
            }
        }
    }
    // Then look at each input, accumulating points as long all the indices match
    input_linestrings
        .par_iter()
        .enumerate()
        .flat_map(|(idx, maybe_linestring)| {
            let mut intermediate_output = Vec::new();

            if let Some(ref linestring) = maybe_linestring {
                // This state is reset as we look through this input's points
                let mut pts_so_far = Vec::new();
                let mut indices_so_far = Vec::new();
                let mut keep_this_output = false;

                for line in linestring.lines() {
                    // The segment is guaranteed to exist
                    let indices = &line_segments[&HashedPoint::new_line(line)];

                    if &indices_so_far == indices {
                        assert_eq!(*pts_so_far.last().unwrap(), line.start);
                        pts_so_far.push(line.end);
                        continue;
                    } else if !pts_so_far.is_empty() {
                        // The overlap ends here
                        let add = Output {
                            geometry: std::mem::take(&mut pts_so_far).into(),
                            indices: std::mem::take(&mut indices_so_far),
                        };
                        if keep_this_output {
                            intermediate_output.push(add);
                        }
                        // Reset below
                    }

                    assert!(pts_so_far.is_empty());
                    pts_so_far.push(line.start);
                    pts_so_far.push(line.end);
                    indices_so_far = indices.clone();
                    // Say we're processing input 2, and we have a segment with indices [2, 5]. We want to
                    // add it to output. But later we'll work on input 5 and see the same segment with
                    // indices [2, 5]. We don't want to add it again, so we'll skip it using the logic
                    // below, since we process input in order.
                    keep_this_output = indices_so_far.iter().all(|i| *i >= idx);
                }

                // This input ended; add to output if needed
                if !pts_so_far.is_empty() && keep_this_output {
                    intermediate_output.push(Output {
                        geometry: pts_so_far.into(),
                        indices: indices_so_far,
                    });
                }
            }

            intermediate_output
        })
        .collect()
}

pub fn feature_to_line_string(f: &Feature) -> Option<geo::LineString<f64>> {
    f.geometry
        .as_ref()
        .and_then(|x| TryInto::<geo::LineString<f64>>::try_into(x).ok())
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
