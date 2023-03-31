use std::collections::HashMap;

use geojson::Feature;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

// TODO Never aggregate across OSM ID threshold. Plumb through an optional property to restrict
// aggregation.

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    #[serde(
        serialize_with = "geojson::ser::serialize_geometry",
        deserialize_with = "geojson::de::deserialize_geometry"
    )]
    pub geometry: geo::LineString<f64>,
    // The indices of Input lines that matched to this segment
    pub indices: Vec<usize>,
}

/// Ignores anything aside from LineStrings. Returns LineStrings chopped up to remove overlap, with
/// exactly one property -- `indices`, an array of numbers indexing the input that share that
/// geometry.
pub fn overline(input: &Vec<Feature>) -> Vec<Output> {
    // For every individual (directed) line segment, record the index of inputs matching there
    let mut line_segments: HashMap<(HashedPoint, HashedPoint), Vec<usize>> = HashMap::new();
    for (idx, input) in input.iter().enumerate() {
        if let Some(geom) = feature_to_line_string(input) {
            for line in geom.lines().map(HashedPoint::new_line) {
                line_segments.entry(line).or_insert_with(Vec::new).push(idx);
            }
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

        if let Some(geom) = feature_to_line_string(input) {
            for line in geom.lines() {
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
                        output.push(add);
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
