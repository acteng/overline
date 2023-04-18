# overline Rust implementation

## How to use it

 `cargo run --release -- --help` to get a summary of options. `cargo run
--release cycle_routes_london.geojson --sum foot --sum bicycle` will produce
`output.geojson`, where features only have a `foot` and `bicycle` property,
summing the values there.

Open
[viewer.html](https://github.com/actenglabs/overline/blob/master/rust/viewer.html)
locally in your browser. You can load an input or output GeoJSON file and
visualize LineStrings, where the width corresponds to the numeric property you
choose.

## Caveats

- Avoid needing overline in the first place, if possible. If your use case is
  summing demand over a transport network and your routing engine can express a
  path as a sequence of edge IDs, do your aggregation using those IDs instead.
  It's much simpler and faster than dealing with geometry.
- Be careful around bridges, tunnels, and other 3D objects. LineStrings are 2D,
  so results may be incorrectly grouped in these cases.
- Only pass in LineStrings coming from the same "upstream" data. Do not use
  this library to match GPS trajectories to a street network or to aggregate a
  bunch of raw GPS trajectories. The input LineStrings are compared exactly; no
  quantization of floating points happens.
