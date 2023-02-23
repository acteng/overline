
# overline

Overline is a function that takes overlapping linestrings and converts
them into a route network, as illustrated in a minimal example below.

![](README_files/figure-commonmark/unnamed-chunk-1-1.png)

![](README_files/figure-commonmark/unnamed-chunk-1-2.png)

    Warning in CPL_write_ogr(obj, dsn, layer, driver,
    as.character(dataset_options), : GDAL Error 6: DeleteLayer() not supported by
    this dataset.

    Warning in CPL_write_ogr(obj, dsn, layer, driver,
    as.character(dataset_options), : GDAL Error 6: DeleteLayer() not supported by
    this dataset.

The function has been implemented in the R package `stplanr` but is
slow, buggy and not feature complete. This repo provides a place to
discuss and develop example code to solve this problem.

In Python, the input and outputs can be visualised as follows:

``` python
import geopandas as gpd
input = gpd.read_file("input.geojson")
input.plot()
```

![](README_files/figure-commonmark/unnamed-chunk-2-1.png)
