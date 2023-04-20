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
# routes$description = integer(routes$description)
rnet = overline(sl = routes, attrib = "description")
sf::write_sf(rnet, "test-data/crossing-routes-minimal-leeds-output.geojson", delete_dsn = TRUE)
# Test total travelled: same for both datasets
sum(sf::st_length(routes) * routes$description)
sum(sf::st_length(rnet) * rnet$description)
# Visualise input and output with tmap:
library(tmap)
m1 = tm_shape(routes) + tm_lines(col = "description", palette = "Blues", scale = 9, breaks = 1:5, as.count = TRUE) + tm_layout(scale = 3)
m2 = tm_shape(rnet) + tm_lines(col = "description", palette = "Blues", scale = 9, breaks = 1:5, as.count = TRUE) + tm_layout(scale = 3)
tmap_arrange(m1, m2, ncol = 2)

# Test with Rust version:
command = "cargo run --manifest-path rust/Cargo.toml test-data/crossing-routes-minimal-leeds.geojson -o test-data/crossing-routes-minimal-leeds-output-rust.geojson --sum description"
system(command)
rnet_rust = sf::read_sf("test-data/crossing-routes-minimal-leeds-output-rust.geojson")
summary(rnet_rust)
tm_shape(rnet_rust) + tm_lines(col = "description", palette = "Blues", scale = 9, breaks = 1:5, as.count = TRUE) + tm_layout(scale = 3, title = "Rust version")

# Test example with 2 lines parallel for some of the way:
library(sf)
library(stplanr)
line1 = sf::st_linestring(matrix(c(0, 1, 0, 0), ncol = 2))
line2 = sf::st_linestring(matrix(c(
    0.2, -0.1,
    0.2, 0,
    0.8, 0,
    0.8, 0.1
    ), ncol = 2, byrow = TRUE))
plot(line1, col = "red", lwd = 9)
plot(line2, col = "blue", lwd = 5, add = TRUE)
lines_without_shared_vertices = sf::st_sfc(line1, line2)
dfr = data.frame(value = c(1, 2))
lines_without_shared_vertices_sf = sf::st_sf(dfr, geometry = lines_without_shared_vertices)
sf::st_crs(lines_without_shared_vertices_sf) = "EPSG:4326"
sf::write_sf(lines_without_shared_vertices_sf, "test-data/lines_without_shared_vertices_sf.geojson", delete_dsn = TRUE)
lines_without_shared_vertices_overline = overline(sl = lines_without_shared_vertices_sf, attrib = "value")
tm_shape(lines_without_shared_vertices_overline) + tm_lines(col = "value", palette = "Blues", scale = 9, breaks = 1:5, as.count = TRUE) + tm_layout(scale = 3, title = "R version")
sf::write_sf(lines_without_shared_vertices_overline, "test-data/lines_without_shared_vertices_overline_r.geojson", delete_dsn = TRUE)

# Test total travelled: same for both datasets
sum(sf::st_length(lines_without_shared_vertices_sf) * lines_without_shared_vertices_sf$value)
sum(sf::st_length(lines_without_shared_vertices_overline) * lines_without_shared_vertices_overline$value)

# Test with two lines that overlap in multiple places:
list.files("rust/tests/")
lines_with_overlaps = sf::read_sf("rust/tests/atip_input.geojson")
lines_with_overlaps_r_overline = stplanr::overline(sl = lines_with_overlaps, attrib = "foot")
sf::write_sf(lines_with_overlaps_r_overline, "test-data/lines_with_overlaps_r_overline.geojson", delete_dsn = TRUE)
lines_with_overlaps_rust_overline = sf::read_sf("rust/tests/atip_output.geojson")
waldo::compare(line)