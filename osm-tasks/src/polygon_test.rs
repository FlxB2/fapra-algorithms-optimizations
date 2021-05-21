pub struct PointInPolygonTest {
    bounding_boxes: Vec<(f64, f64, f64, f64)>,
    polygons: Vec<Vec<(f64, f64)>>,
}

pub struct Point(f64, f64);

impl Point {
    fn new(lon: f64, lat: f64) -> Point {
        Point(lat, lon)
    }
    fn from((lon, lat): &(f64, f64)) -> Point {
        Point(*lat, *lon)
    }

    fn lat(&self) -> f64 { self.0 }
    fn lon(&self) -> f64 { self.1 }
}

impl PointInPolygonTest {
    pub fn new(polygons: Vec<Vec<(f64, f64)>>) -> PointInPolygonTest {
        // println!("Polygon test instance with {} polygons", polygons.len());
        let bounding_boxes: Vec<(f64, f64, f64, f64)> = polygons.iter().map(|polygon| PointInPolygonTest::calculate_bounding_box(polygon)).collect();
        return PointInPolygonTest { bounding_boxes, polygons };
    }

    fn check_point_between_edges((point_lon, point_lat): &(f64, f64), (v1_lon, v1_lat): &(f64, f64), (v2_lon, v2_lat): &(f64, f64)) -> bool {
        if v1_lon == v2_lon {
            // Ignore north-south edges
            return false;
        } else if v1_lat == v2_lat {
            return f64::min(*v1_lon, *v2_lon) <= *point_lon && *point_lon <= f64::max(*v1_lon, *v2_lon);
        } else if *point_lon < f64::min(*v1_lon, *v2_lon) || f64::max(*v1_lon, *v2_lon) < *point_lon {
            // Can not intersect with the edge
            return false;
        }
        // Todo: If both ends of the edge are in the northern hemisphere and the test point is south of the chord (on a lat-Ion projection) between the end points, it intersects the edge.

        let v1_lon_rad = v1_lon.to_radians();
        let v1_lat_tan = v1_lat.to_radians().tan();
        let v2_lon_rad = v2_lon.to_radians();
        let v2_lat_tan = v2_lat.to_radians().tan();
        let delta_v_lon_sin = (v1_lon_rad - v2_lon_rad).sin();
        let point_lon_rad = point_lon.to_radians();

        let intersection_lat_tan = (v1_lat_tan * ((point_lon_rad - v2_lon_rad).sin() / delta_v_lon_sin) - v2_lat_tan * ((point_lon_rad - v1_lon_rad).sin() / delta_v_lon_sin));
        if intersection_lat_tan == v1_lat_tan || intersection_lat_tan == v2_lat_tan {
            //special case: intersection is on one of the vertices
            let (hit_vert_lon_rad, other_vert_lon_rad) = if intersection_lat_tan == v1_lat_tan { (v1_lon_rad, v2_lon_rad) } else { (v2_lon_rad, v1_lon_rad) };
            // tread it as in polygon iff the other vertex is westward of the hit vertex
            return (hit_vert_lon_rad - other_vert_lon_rad).sin() > 0f64;
        }

        // intersection must be between the vertices and not below the point
        f64::min(v1_lat_tan, v2_lat_tan) <= intersection_lat_tan
            && intersection_lat_tan <= f64::max(v1_lat_tan, v2_lat_tan)
            && intersection_lat_tan >= point_lat.to_radians().tan()
    }

    fn calculate_bounding_box(polygon: &Vec<(f64, f64)>) -> (f64, f64, f64, f64) {
        let mut lon_min = 180_f64;
        let mut lon_max = -180_f64;
        let mut lat_min = 180_f64;
        let mut lat_max = -180_f64;
        for (lon, lat) in polygon {
            lon_min = f64::min(lon_min, *lon);
            lon_max = f64::max(lon_max, *lon);
            lat_min = f64::min(lat_min, *lat);
            lat_max = f64::max(lat_max, *lat);
        }
        //println!("Bounding Box: ({},{}) to ({},{})", lon_min, lat_min, lon_max, lat_max);
        (lon_min, lon_max, lat_min, lat_max)
    }

    fn check_intersecting_bounding_boxes(&self, (lon, lat): (f64, f64)) -> Vec<usize> {
        let mut matching_polygons: Vec<usize> = Vec::with_capacity(self.polygons.len());
        self.bounding_boxes.iter().enumerate().for_each(|(idx, (lon_min, lon_max, lat_min, lat_max))| {
            if lon >= *lon_min && lon <= *lon_max && lat >= *lat_min && lat <= *lat_max {
                matching_polygons.push(idx);
                //println!("Point ({},{}) is inside bounding box of polygon {}", lon, lat, idx);
            }
        });
        matching_polygons.shrink_to_fit();
        return matching_polygons;
    }

    fn check_point_in_polygons(&self, (point_lon, point_lat): (f64, f64), polygon_indices: Vec<usize>) -> bool {
        let mut intersection_count_even = true;
        //let mut intersections: Vec<((f64, f64), (f64, f64))> = vec![];
        for polygon_idx in polygon_indices {
            intersection_count_even = true;
            let polygon = &self.polygons[polygon_idx];
            for i in 0..polygon.len() - 1 {
                if polygon[i].1 < point_lat && polygon[i + 1].1 < point_lat {
                    continue;
                }
                if polygon[i] == (point_lon, point_lat) {
                    // Point is at the vertex -> we define this as within the polygon
                    return true;
                }
                if PointInPolygonTest::check_point_between_edges(&(point_lon, point_lat), &polygon[i], &polygon[i + 1]) {
                    intersection_count_even = !intersection_count_even;
                    //  intersections.push((polygon[i], polygon[i + 1]));
                }
            }
            if !intersection_count_even {
                break;
            }
        }
        //write_to_file("lines".parse().unwrap(), lines_to_json(intersections));
        return !intersection_count_even;
    }
    const EARTH_RADIUS: i32 = 6_378_137;

    fn calculate_length_between_points(p1: &Point, p2: &Point) -> f64 {
        PointInPolygonTest::EARTH_RADIUS as f64 * ((p2.lon() - p1.lon()).powi(2) * ((p1.lat() + p2.lat()) / 2f64).cos().powi(2) * (p2.lat() - p1.lat()).powi(2)).sqrt()
    }

    pub fn check_intersection(&self, point: (f64, f64)) -> bool {
        // first get all intersecting bounding boxes
        let polygons_to_check = self.check_intersecting_bounding_boxes(point.clone());
        // check these polygons with point in polygon test
        self.check_point_in_polygons(point, polygons_to_check)
    }
}
