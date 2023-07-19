
# overline

Overline is a function that takes overlapping linestrings and converts
them into a route network (Morgan and Lovelace 2020) as illustrated in a
minimal reproducible example created with
[ATIP](https://github.com/acteng/atip/).

The input is a dataset in which some segments overlap:

    Reading layer `minimal_2_lines' from data source 
      `/home/robin/github/acteng/overline/test-data/minimal_2_lines.txt' 
      using driver `GeoJSON'
    Simple feature collection with 2 features and 5 fields
    Geometry type: LINESTRING
    Dimension:     XY
    Bounding box:  xmin: -1.29569 ymin: 50.69972 xmax: -1.295337 ymax: 50.69987
    Geodetic CRS:  WGS 84

![](README_files/figure-commonmark/unnamed-chunk-1-1.png)

The output is a dataset in which the overlapping segments have been
combined:

![](README_files/figure-commonmark/unnamed-chunk-2-1.png)

The functionality has been implemented in the [`overline()`
function](https://docs.ropensci.org/stplanr/reference/overline.html) in
the R package `stplanr`. The function works fine for city sized datasets
but for national datasets is slow, buggy and not feature complete, as it
does not retain OSM IDs. This repo provides a place to discuss and
develop example code to solve this problem.

In Python, the input and outputs can be visualised as follows:

``` python
import geopandas as gpd
input = gpd.read_file("input.geojson")
# Plot with colour by value:
input.plot(column="value")
```

![](README_files/figure-commonmark/unnamed-chunk-3-1.png)

``` python
output = gpd.read_file("output.geojson")
output.plot(column="value")
```

![](README_files/figure-commonmark/unnamed-chunk-4-3.png)

![](README_files/figure-commonmark/unnamed-chunk-3-3.png)

# Example with road names

The example below takes routes at the segment level and calculates
average gradient for each segment. Road names are NOT currently
implemented in `overline()` in R.

``` r
sl_desire_lines = stplanr::flowlines_sf[2:3, ]
# qtm(sl_desire_lines) +
#   qtm(sl)
route_segments_minimal = stplanr::route(
  l = sl_desire_lines,
  route_fun = cyclestreets::journey
  )
```

    Most common output is sf

``` r
names(route_segments_minimal)
```

     [1] "Area.of.residence"                   
     [2] "Area.of.workplace"                   
     [3] "All"                                 
     [4] "Work.mainly.at.or.from.home"         
     [5] "Underground..metro..light.rail..tram"
     [6] "Train"                               
     [7] "Bus..minibus.or.coach"               
     [8] "Taxi"                                
     [9] "Motorcycle..scooter.or.moped"        
    [10] "Driving.a.car.or.van"                
    [11] "Passenger.in.a.car.or.van"           
    [12] "Bicycle"                             
    [13] "On.foot"                             
    [14] "Other.method.of.travel.to.work"      
    [15] "id"                                  
    [16] "route_number"                        
    [17] "name"                                
    [18] "distances"                           
    [19] "time"                                
    [20] "busynance"                           
    [21] "quietness"                           
    [22] "gradient_segment"                    
    [23] "elevation_change"                    
    [24] "provisionName"                       
    [25] "start_longitude"                     
    [26] "start_latitude"                      
    [27] "finish_longitude"                    
    [28] "finish_latitude"                     
    [29] "crow_fly_distance"                   
    [30] "event"                               
    [31] "whence"                              
    [32] "speed"                               
    [33] "itinerary"                           
    [34] "plan"                                
    [35] "note"                                
    [36] "length"                              
    [37] "west"                                
    [38] "south"                               
    [39] "east"                                
    [40] "north"                               
    [41] "leaving"                             
    [42] "arriving"                            
    [43] "grammesCO2saved"                     
    [44] "calories"                            
    [45] "edition"                             
    [46] "gradient_smooth"                     
    [47] "geometry"                            

``` r
tm_shape(route_segments_minimal) +
  tm_lines("name")
```

![](README_files/figure-commonmark/unnamed-chunk-5-5.png)

``` r
rnet_from_cyclestreets = overline(
  route_segments_minimal,
  attrib = c("All", "gradient_smooth", "quietness"),
  fun = c(sum = sum, mean = mean)
  )
rnet_from_cyclestreets = rnet_from_cyclestreets %>% 
  transmute(All = All_sum, Gradient = gradient_smooth_mean, Quietness = quietness_mean)
plot(rnet_from_cyclestreets)
```

![](README_files/figure-commonmark/unnamed-chunk-5-6.png)

``` r
sf::write_sf(route_segments_minimal, "route_segments_minimal.geojson", delete_dsn = TRUE)
sf::write_sf(rnet_from_cyclestreets, "rnet_from_cyclestreets.geojson", delete_dsn = TRUE)
```

# Large example

A large example plus benchmark is shown below:

``` r
# list.files()
cycle_routes_london = pct::get_pct_routes_fast("london")
sf::write_sf(cycle_routes_london, "cycle_routes_london.geojson")
zip("cycle_routes_london.zip", "cycle_routes_london.geojson")
system("gh release upload v0 cycle_routes_london.zip")
```

``` r
system.time({
  cycle_routes_london = geojsonsf::geojson_sf("cycle_routes_london.geojson")
  names(cycle_routes_london)
  rnet = overline(cycle_routes_london, attrib = "foot")
})
# sf::write_sf(rnet, "rnet_london.geojson")
# system("gh release upload v0 rnet_london.geojson")
```

The operation took around 2 minutes.

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
