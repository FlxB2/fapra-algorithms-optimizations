use std::fs::File;
use std::io::Write;

/*
generates a file following the GeoJSON format https://datatracker.ietf.org/doc/html/rfc7946
 */
pub struct JsonFile {
    file_name: String,
    polygons: Vec<Vec<(f64, f64)>>,
    // polygon in geojson
    points: Vec<(f64, f64)>, // multipoint in geojson
}

impl JsonFile {
    pub fn to_string(&self) -> String {
        let mut result = String::new() + "  { \"type\": \"FeatureCollection\",
            \"features\": [";

        // polygons
        for polygon in &self.polygons {
            if polygon.len() > 1 {
                // ensure first equals last ndoe
                let mut temp = polygon.to_vec();
                temp.push(*polygon.first().unwrap());

                let mut coords_string = format!("{:?}", temp).replace("(", "[").replace(")", "]");
                coords_string = format!("{{
              \"type\": \"Feature\",
              \"properties\": {{}},
              \"geometry\": {{
                \"type\": \"Polygon\",
                \"coordinates\": [
                    {}
                ]
              }}
             }},", coords_string);
                result = result + &*coords_string;
            }
        }
        // mutli points
        if self.points.len() > 0 {
            let mut p = self.points.to_vec();
            let mut coords_string = format!("{:?}", p).replace("(", "[").replace(")", "]");
           // println!("{}", coords_string);
            coords_string = format!("{{
              \"type\": \"Feature\",
              \"properties\": {{}},
              \"geometry\": {{
                \"type\": \"MultiPoint\",
                \"coordinates\":
                    {}
              }}
             }},", coords_string);
            result = result + &*coords_string;
        }
        result = result + "]}";
        result
    }
}

pub struct JsonBuilder {
    json: JsonFile,
}

impl JsonBuilder {
    pub fn new(file_name: String) -> JsonBuilder {
        let file = JsonFile { file_name, polygons: Vec::new(), points: Vec::new() };
        JsonBuilder {
            json: file
        }
    }

    pub fn add_polygon(&mut self, polygon: Vec<(f64, f64)>) -> &mut JsonBuilder {
        self.json.polygons.push(polygon);
        self
    }

    pub fn add_polygons(&mut self, polygons: Vec<Vec<(f64, f64)>>) -> &mut JsonBuilder {
        self.json.polygons.extend(polygons);
        self
    }

    pub fn add_points(&mut self, points: Vec<(f64, f64)>) -> &mut JsonBuilder {
        self.json.points.extend(points);
        self
    }

    pub fn build(&mut self) -> File {
        let mut file = File::create("poly").expect("could not open file");
        file.write_all(self.json.to_string().as_ref()).expect("could not write to file");
        file
    }
}
