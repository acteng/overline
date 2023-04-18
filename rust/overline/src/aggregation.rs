use geojson::Feature;
use ordered_float::NotNan;
use rayon::prelude::*;

use crate::Output;

#[derive(Clone, Copy)]
pub enum Aggregation {
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

/// `grouped_indices` is the result of `overline(input)`. This copies new Features, filling out
/// the listed properties as specified.
pub fn aggregate_properties(
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
