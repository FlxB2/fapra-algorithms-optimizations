fn main() {
    println!("Hello, world!");
    use osmpbf::{ElementReader, Element};

    let reader = ElementReader::from_path("./monaco-latest.osm.pbf").expect("failed");
    let mut ways = 0_u64;

// Increment the counter by one for each way.
    reader.for_each(|element| {
        if let Element::Way(_) = element {
            ways += 1;
        }
    }).expect("failed5");
}
