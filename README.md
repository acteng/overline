
# overline

Overline is a function that takes overlapping linestrings and converts
them into a route network (Morgan and Lovelace 2020) as illustrated in a
minimal reproducible example below.

``` r
library(sf)
library(stplanr)
sl = routes_fast_sf[2:3, 0]
sl$n = 1:2
plot(sl)
```

![](README_files/figure-commonmark/unnamed-chunk-1-1.png)

``` r
rnet = overline(sl, attrib = "n")
plot(rnet)
```

![](README_files/figure-commonmark/unnamed-chunk-1-2.png)

``` r
sf::write_sf(sl, "minimal-example-input.geojson", delete_dsn = TRUE)
sf::write_sf(rnet, "minimal-example-output.geojson", delete_dsn = TRUE)
```

The function has been implemented in the [`overline()`
function](https://docs.ropensci.org/stplanr/reference/overline.html) in
the R package `stplanr`. The function works fine for city sized datasets
but for national datasets is slow, buggy and not feature complete, as it
does not retain OSM IDs. This repo provides a place to discuss and
develop example code to solve this problem.

In Python, the input and outputs can be visualised as follows:

``` python
import geopandas as gpd
input = gpd.read_file("input.geojson")
input.plot()
```

![](README_files/figure-commonmark/unnamed-chunk-2-1.png)

``` python
output = gpd.read_file("output.geojson")
output.plot()
```

![](README_files/figure-commonmark/unnamed-chunk-3-3.png)

# References

<div id="refs" class="references csl-bib-body hanging-indent">

<div id="ref-morgan2020" class="csl-entry">

Morgan, Malcolm, and Robin Lovelace. 2020. “Travel Flow Aggregation:
Nationally Scalable Methods for Interactive and Online Visualisation of
Transport Behaviour at the Road Network Level.” *Environment and
Planning B: Urban Analytics and City Science* 48 (6): 1684–96.
<https://doi.org/10.1177/2399808320942779>.

</div>

</div>
