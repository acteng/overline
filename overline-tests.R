library(sf)
library(stplanr)
# ?stplanr::overline
sl <- routes_fast_sf[2:4, ]
sl$All <- flowlines_sf$All[2:4]
rnet <- overline(sl = sl, attrib = "All")
nrow(sl)
nrow(rnet)
plot(rnet)

sf::write_sf(sl, "input.geojson")
sf::write_sf(rnet, "output.geojson")

# Test total travelled
sum(sf::st_length(sl) * sl$All)
sum(sf::st_length(rnet) * rnet$All)

# Command line: