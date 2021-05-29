use geo::{Polygon, Rect};
use quadtree_rs::{area::AreaBuilder, point::Point as qPoint, Quadtree};
use crate::kml_exporter::KML_export;

pub struct PointInPolygonTest {
    bounding_boxes: Vec<(f64, f64, f64, f64)>,
    polygons: Vec<Vec<(f64, f64)>>,
    quadtree: Quadtree<i16, i32>,
    grid: Option<Vec<GridEntry>>
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

/**
Point in Polygon test
Uses three stages to check if a point is inside a polygon:
1.  Use a quadtree to determine the polygons that are in the same region as the point.
    The quadtree uses a resolution of integral lat lon coordinates.
2. Use lat lon aligned bounding boxes to further narrow down the potential polygons which could be hit by the point
3. Do the actual point in polygon test
**/
impl PointInPolygonTest {
    pub fn new(polygons: Vec<Vec<(f64, f64)>>) -> PointInPolygonTest {
        // println!("Polygon test instance with {} polygons", polygons.len());
        let bounding_boxes: Vec<(f64, f64, f64, f64)> = polygons.iter().map(|polygon| PointInPolygonTest::calculate_bounding_box(polygon)).collect();
        let quadtree = PointInPolygonTest::build_quadtree(&bounding_boxes);
        let mut polygon_test = PointInPolygonTest { bounding_boxes, polygons, quadtree, grid: None };
        polygon_test.build_grid();
        return polygon_test;
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
        let mut lat_min = 90_f64;
        let mut lat_max = -90_f64;
        for (lon, lat) in polygon {
            lon_min = f64::min(lon_min, *lon);
            lon_max = f64::max(lon_max, *lon);
            lat_min = f64::min(lat_min, *lat);
            lat_max = f64::max(lat_max, *lat);
        }
        //println!("Bounding Box: ({},{}) to ({},{})", lon_min, lat_min, lon_max, lat_max);
        (lon_min, lon_max, lat_min, lat_max)
    }

    fn build_quadtree(bounding_boxes: &Vec<(f64, f64, f64, f64)>) -> Quadtree<i16, i32> {
        let mut quadtree = Quadtree::<i16, i32>::new(9);
        for i in 0..bounding_boxes.len() {
            let bounding_box = bounding_boxes[i];
            let x = bounding_box.0.floor() as i16;
            let y = bounding_box.2.floor() as i16;
            let x_size = bounding_box.1.ceil() as i16 - x;
            let y_size = bounding_box.3.ceil() as i16 - y;
            let res = quadtree.insert(AreaBuilder::default()
                                          .anchor(qPoint { x: x + 180i16, y: y + 90i16 })
                                          .dimensions((x_size, y_size))
                                          .build().unwrap(), i as i32);
            //println!("{:?}", res);
        }
        //println!("{:?}", quadtree);
        quadtree
    }

    fn build_grid(&mut self) {
        // In the beginning, there is only water on the whole world.
        let mut grid = vec![GridEntry::Outside; 360 * 180];
        for i in 0..self.bounding_boxes.len() {
            let bounding_box = self.bounding_boxes[i];
            let polygon = &self.polygons[i];
            let x = bounding_box.0.floor() as i16;
            let y = bounding_box.2.floor() as i16;
            let x_size = bounding_box.1.floor() as i16 + 1 - x;
            let y_size = bounding_box.3.floor() as i16 + 1 - y;
            let mut rects_with_points = vec![RectState::Initial; (x_size * y_size) as usize];
            // find rects containing points
            for (lon, lat) in polygon {
                if *lon as i16 == 180 {
                    // thread this point as it would be in the 179 rect. This works because this
                    // is most likely a point of a polygon that was spilt at the 180 degree line.
                    // So its counterpart with the .-180 degree point will eventually processed
                    rects_with_points[((179 - x) + ((lat.floor() as i16 - y) * x_size)) as usize] = RectState::ContainsPoints;
                    continue
                }
                //println!("lon {}, let {}, index poly {}, x {}, y {}, xsize {}, ysize {}", lon, lat, i,x,y,x_size,y_size );
                rects_with_points[((lon.floor() as i16 - x) + ((lat.floor() as i16 - y) * x_size)) as usize] = RectState::ContainsPoints;
            }
            // Iterate over the the grid and process every rect
            for r_y in 0..y_size {
                for r_x in 0..x_size {
                    if rects_with_points[(r_x + (r_y * x_size)) as usize] != RectState::Initial {
                        if rects_with_points[(r_x + (r_y * x_size)) as usize] == RectState::ContainsPoints {
                            PointInPolygonTest::insert_in_grid(&mut grid, GridEntry::Border, r_x + x, r_y + y);
                        }
                        continue;
                    }
                    let points = PointInPolygonTest::mark_coherent_rects(&mut rects_with_points, r_x, r_y, x_size as usize, y_size as usize);
                    // test if this coherent block of points is part of the polygon
                    if let Some(some_point) = points.first() {
                        let ps_y = *some_point as i16 / x_size;
                        let ps_x = *some_point as i16 - (ps_y * x_size);
                        let entry = if self.check_point_in_polygons(((ps_x + x) as f64 + 0.5, (ps_y + y) as f64 + 0.5), vec![i]) { GridEntry::Polygon } else { GridEntry::Outside };
                        points.into_iter().for_each(|idx| {
                            let p_y = idx as i16 / x_size;
                            let p_x = idx as i16 - (p_y * x_size);
                            PointInPolygonTest::insert_in_grid(&mut grid, entry, p_x + x, p_y + y);
                            //kml.rect( p_x + x , p_y + y, Some(if entry == GridEntry::Polygon { "polygon" } else { "outside" }.parse().unwrap()));
                        });
                    }
                }
            }
        }
        /*
        let mut kml_poly = KML_export::init();
        let mut kml_outside = KML_export::init();
        for idx in 0..grid.len(){
            if grid[idx] == GridEntry::Polygon {
                let p_y = idx as i16 / 360;
                let p_x = idx as i16 - (p_y * 360);
                kml_poly.rect( p_x - 180 , p_y - 90, Some(format!("Poly:{}", idx)));
            } else if grid[idx] == GridEntry::Outside {
                let p_y = idx as i16 / 360;
                let p_x = idx as i16 - (p_y * 360);
                kml_outside.rect( p_x - 180 , p_y - 90, Some(format!("Outside:{}", idx)));
            }
        }
        kml_poly.write_file("poly_rects.kml".parse().unwrap());
        kml_outside.write_file("outside_rects.kml".parse().unwrap());
         */
        self.grid = Some(grid);
    }


    fn insert_in_grid(grid: &mut Vec<GridEntry>, entry: GridEntry, x: i16, y: i16) {
        let idx = ((x + 180) as usize + ((y + 90) as usize * 360));
        if entry == GridEntry::Border {
            // border has to be checked every time
            grid[idx] = entry;
            return;
        }
        if grid[idx] == GridEntry::Outside && entry == GridEntry::Polygon {
            grid[idx] = entry;
        }
    }

    fn mark_coherent_rects(rects_with_points: &mut Vec<RectState>, start_x: i16, start_y: i16, x_size: usize, y_size: usize) -> Vec<usize> {
        if rects_with_points[(start_x + (start_y * x_size as i16)) as usize] != RectState::Initial {
            return vec![];
        }
        let mut coherent_rects = Vec::with_capacity(rects_with_points.len());
        let mut queue = Vec::with_capacity(10);
        // traverse all
        queue.push((start_x + (start_y * x_size as i16)) as usize);
        while !queue.is_empty() {
            let idx = queue.pop().unwrap();
            coherent_rects.push(idx);
            let y = idx / x_size;
            let x = idx - (y * x_size);
            // add neighbors to queue
            if x > 0 {
                PointInPolygonTest::process_rect_node(&mut queue, rects_with_points, x_size, x - 1, y);
            }
            if x < x_size - 1 {
                PointInPolygonTest::process_rect_node(&mut queue, rects_with_points, x_size, x + 1, y);
            }
            if y > 0 {
                PointInPolygonTest::process_rect_node(&mut queue, rects_with_points, x_size, x, y - 1);
            }
            if y < y_size - 1 {
                PointInPolygonTest::process_rect_node(&mut queue, rects_with_points, x_size, x, y + 1);
            }
        }
        // collected all coherent rects
        //println!("{:?}", coherent_rects);
        coherent_rects
    }

    #[inline]
    fn process_rect_node(queue: &mut Vec<usize>, rects_with_points: &mut Vec<RectState>, x_size: usize, x: usize, y: usize) {
        let idx = x + (y * x_size);
        if rects_with_points[idx] == RectState::Initial {
            queue.push(idx);
            rects_with_points[idx] = RectState::Processed;
        }
    }

    fn check_intersecting_bounding_boxes(&self, (lon, lat): (f64, f64)) -> Vec<usize> {
        let mut matching_polygons: Vec<usize> = Vec::with_capacity(self.polygons.len());
        // find potential polygons with the quadtree
        self.quadtree.query(AreaBuilder::default()
            .anchor(qPoint { x: lon.floor() as i16 + 180i16, y: lat.floor() as i16 + 90i16 })
            .dimensions((1, 1))
            .build().unwrap())
            .for_each(|e| {
                //println!("Quadtree bounding box intersection");
                let idx = *e.value_ref() as usize;
                // do bounding box test for polygons in this quadrant of the quadtree
                let (lon_min, lon_max, lat_min, lat_max) = self.bounding_boxes.get(idx).unwrap();
                if lon >= *lon_min && lon <= *lon_max && lat >= *lat_min && lat <= *lat_max {
                    matching_polygons.push(idx);
                    //println!("Point ({},{}) is inside bounding box of polygon {}", lon, lat, idx);
                }
            });
        matching_polygons.shrink_to_fit();
        return matching_polygons;
    }

    fn check_grid(&self, (lon, lat): (f64, f64)) -> &GridEntry {
        if self.grid.is_none() {
            return &GridEntry::Border;
        }
        return self.grid.as_ref().unwrap().get(((lon.floor() as i16 + 180) as usize + ((lat.floor() as i16 + 90) as usize * 360))).unwrap();
    }

    fn check_point_in_polygons(&self, (mut point_lon, point_lat): (f64, f64), polygon_indices: Vec<usize>) -> bool {
        if point_lat < -85.11 {
            // hit south pole, but there is no polygon
            // this check is part of this method since this is used at the generation of the grid
            return true;
        }
        if point_lon as i16 == 180 {
            println!("Point at 180. Map to -180: ({}, {})", point_lon, point_lat);
            point_lon = -180.0;
        }
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
                if polygon[i].0 == polygon[i+1].0 && polygon[i].0 == point_lon  {
                    // MM: fixed
                    // north south edge. Check if the point is on this edge
                    if polygon[i].1.min(polygon[i+1].1) < point_lat && polygon[i].1.max(polygon[i+1].1) > point_lat {
                        // point on this edge
                        return true;
                    }
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
        if point.1 < -85.11 {
            // hit south pole, but there is no polygon
            return true;
        }
        // shortcut: First check grid
        let gridEntry = self.check_grid(point.clone());
        if *gridEntry == GridEntry::Polygon || *gridEntry == GridEntry::Outside {
            return *gridEntry == GridEntry::Polygon;
        }
        // first get all intersecting bounding boxes
        let polygons_to_check = self.check_intersecting_bounding_boxes(point.clone());
        // check these polygons with point in polygon test
        self.check_point_in_polygons(point, polygons_to_check)
    }
    pub fn polygons(&self) -> &Vec<Vec<(f64, f64)>> {
        &self.polygons
    }
}
#[derive(Clone, PartialEq, Eq, Copy)]
enum RectState {
    Initial,
    ContainsPoints,
    Processed,
}
#[derive(Clone, PartialEq, Eq, Copy)]
enum GridEntry {
    Polygon,
    Outside,
    Border
}
