library(sf)
library(stplanr)
# ?stplanr::overline
sl = routes_fast_sf[2:4, ]
sl$All = flowlines_sf$All[2:4]
rnet = overline(sl = sl, attrib = "All")
nrow(sl)
nrow(rnet)
plot(rnet)

sf::write_sf(sl, "test-data/input-minimal.geojson")
sf::write_sf(rnet, "test-data/output-minmal.geojson")

# Test total travelled
sum(sf::st_length(sl) * sl$All)
sum(sf::st_length(rnet) * rnet$All)

# Test an example with 2 lines parallel for some of the way:
library(sf)
routes = sf::read_sf("test-data/crossing-routes-minimal-leeds.geojson")
routes$description = integer(routes$description)
rnet = overline(sl = routes, attrib = "description")
sf::write_sf(rnet, "test-data/crossing-routes-minimal-leeds-output.geojson")
# Test total travelled: same for both datasets
sum(sf::st_length(routes) * routes$description)
sum(sf::st_length(rnet) * rnet$description)
# Visualise input and output with tmap:
library(tmap)
m1 = tm_shape(routes) + tm_lines(col = "description", palette = "Blues", scale = 9, breaks = 1:5, as.count = TRUE) + tm_layout(scale = 3)
m2 = tm_shape(rnet) + tm_lines(col = "description", palette = "Blues", scale = 9, breaks = 1:5, as.count = TRUE) + tm_layout(scale = 3)
tmap_arrange(m1, m2, ncol = 2)
