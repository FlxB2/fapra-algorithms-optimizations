use std::collections::HashMap;
use std::fs::File;

use kml::{Kml, KmlDocument, KmlWriter, quick_collection, types::{AltitudeMode, Coord, LineString, Point, Polygon}};
use kml::types::{Geometry, LinearRing, Placemark};

pub struct KML_export {
    elements: Vec<Kml<f64>>,
}

impl KML_export {
    pub fn init() -> KML_export {
        return KML_export {
            elements: vec![]
        };
    }

    fn as_placemarker(name: Option<String>, geometry: Geometry) -> Kml {
        return Kml::Placemark(Placemark {
            name,
            description: None,
            geometry: Some(geometry),
            attrs: HashMap::new(),
            children: vec![],
        }
        );
    }

    pub fn convert_coords(points: Vec<(f64, f64)>) -> Vec<Coord> {
        points.into_iter().map(|(lon, lat)| { Coord { x: lon, y: lat, z: None } }).collect()
    }

    pub fn add_polygon(&mut self, polygon: Vec<(f64, f64)>, name: Option<String>) {
        let points: Vec<Coord> = KML_export::convert_coords(polygon);
        self.elements.push(KML_export::as_placemarker(name, Geometry::Polygon(Polygon::new(LinearRing::from(points), vec![]))));
    }

    pub fn add_point(&mut self, point: (f64, f64), name: Option<String>) {
        self.elements.push(KML_export::as_placemarker(name, Geometry::Point(Point::new(point.0, point.1, None))));
    }

    pub fn add_linestring(&mut self, line: Vec<(f64, f64)>, name: Option<String>) {
        let points: Vec<Coord> = KML_export::convert_coords(line);
        self.elements.push(KML_export::as_placemarker(name, Geometry::LineString(LineString::from(points))));
    }

    pub fn write_file(&self, path: String) {
        let mut file = File::create(path).unwrap();
        let mut writer = KmlWriter::<_, f64>::from_writer(&mut file);
        let kml = Kml::Document {
            attrs: HashMap::new(),
            elements: self.elements.clone(),
        };
        writer.write(&kml);
    }
}