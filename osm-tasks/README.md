# OsmTasksBackend

## Prerequisites

The program requires `rust` and `cargo`.
An installation of rust and cargo, is described in this [guide](https://www.rust-lang.org/tools/install).

Since the program uses rocket, a nightly build of rust is required to build the program.
A nightly version of rust, can be installed via `rustup default nightly`. Additional information about how to install a nightly build can be found at the [Rocket Guide](https://rocket.rs/v0.4/guide/getting-started/).

## Setup
To build the program you can either use `cargo build --release` to compile the program.
Afterward the binary can started with `./target/release/osm-tasks <OSM coastlines file> -b -n <node number>`

Alternatively the program can be started directly via cargo. To do so, use the command:

`cargo run --release -- <OSM coastlines file> -b -n <node number>`.

## Command line Arguments

The program supports a few command line arguments.
Use `--help` to show all available command line arguments:
```
USAGE:
    osm-tasks [FLAGS] [OPTIONS] <coastlines-file>

ARGS:
    <coastlines-file>    Coastlines file used to generate the polygons

FLAGS:
    -b, --build-graph-on-startup    Build graph on startup. Sets wether the graph generation should
                                    be triggered at startup. Generation trough REST API will be
                                    available anyway
    -f, --force-rebuild-graph       Set this if the graph should be generated from scratch. If this
                                    is not set, the program will try to load an already generated
                                    graph of the form <coastlines_file>.<number_of_nodes>.bin
    -h, --help                      Prints help information           
    -V, --version                   Prints version information

OPTIONS:
    -g, --geojson-export-path <geojson-export-path>
            Path to save the exported geoJSON file with the generated polygons, if the polygons
            should be exported. If no file is specified, the file is not generated

    -n, --nodes <number-of-nodes>
            Number of points which will equaly distributed over the sphere. Each point outside of a
            polygon will generate a node in the graph. So this is the upper bound for the number of
            nodes in the graph [default: 10000]
```
Use the '-n <node number>' to set the number of nodes used for building the graph.
After building the graph, the program will save the graph to disk into a file with the name `<coastlines_file>.<number_of_nodes>.bin`, which will be loaded at further program starts, if the same number of nodes and the same coastlines file (name) is used (unless the `-f` flag is used to ignore the file and rebuild the graph).
## OpenAPI Specification

We used [OpenAPI 3](https://swagger.io/specification/) to specify the API interfaces between the backend and the frontend. The specification file can be found at `http://localhost:8000/openapi.json`
