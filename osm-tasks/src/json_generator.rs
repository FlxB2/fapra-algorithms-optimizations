use std::fs::File;
use std::io::Write;

pub struct JsonFile {
    file_name: String,
    polygons: Vec<Vec<(f64, f64)>>,
}

impl JsonFile {
    pub fn to_string(&self) -> String {
        let mut result = String::new();

        for polygon in &self.polygons {
            let mut coords_string = format!("{:?}", polygon).replace("(", "[").replace(")", "]");
            coords_string = format!("{{
              \"type\": \"Feature\",
              \"properties\": {{}},
              \"geometry\": {{
                \"type\": \"Polygon\",
                \"coordinates\": [
                    {}
                ]
              }}
             }}", coords_string);
            result = result + &*coords_string;
        }

        result
    }
}

pub struct JsonBuilder {
    json: JsonFile,
}

impl JsonBuilder {
    pub fn new(file_name: String) -> JsonBuilder {
        let file = JsonFile { file_name, polygons: Vec::new() };
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

    pub fn build(&mut self) -> File {
        let mut file = File::create("poly").expect("could not open file");
        file.write_all(self.json.to_string().as_ref()).expect("could not write to file");
        file
    }
}