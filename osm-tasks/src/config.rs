use clap::Clap;
use once_cell::sync::OnceCell;
use std::path::Path;

static INSTANCE: OnceCell<Config> = OnceCell::new();

/// This doc string acts as a help message when the user runs '--help'
/// OSM FaPra ship routing server
#[derive(Clap, Debug)]
#[clap(version = "1.0")]
pub struct Config {
    /// Path to save the exported geoJSON file with the generated polygons, if the polygons should be exported. If no file is specified, the file is not generated.
    #[clap(short, long)]
    geojson_export_path: Option<String>,

    /// Coastlines file used to generate the polygons
    #[clap()]
    coastlines_file: String,

    /// Set this if the graph should be generated from scratch. If this is not set, the program will try to load an already generated graph of the form <coastlines_file>.<number_of_nodes>.bin
    #[clap(short, long)]
    force_rebuild_graph: bool,

    /// Number of points which will equaly distributed over the sphere. Each point outside of a polygon will generate a node in the graph. So this is the upper bound for the number of nodes in the graph.
    #[clap(short = 'n', long = "nodes", default_value = "10000")]
    number_of_nodes: u32,

    /// Build graph on startup. Sets wether the graph generation should be triggered at startup. Generation trough REST API will be available anyway.
    #[clap(short, long)]
    build_graph_on_startup: bool,

    #[clap(long="max-test")]
    max_test: bool,

    // Todo: Option for KML export

}

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("Config is not initialized")
    }

    pub fn init() {
        if INSTANCE.get().is_some() {
            println!("Config is already loaded!")
        }
        let config = Config::parse();
        // verify paths
        if !Path::new(config.coastlines_file()).is_file() {
            panic!("Could not open coastlines file: {}", config.coastlines_file());
        }
        INSTANCE.set(config).unwrap();
    }

    pub fn coastlines_file(&self) -> &str {
        &self.coastlines_file
    }
    pub fn force_rebuild_graph(&self) -> bool {
        self.force_rebuild_graph
    }
    pub fn number_of_nodes(&self) -> u32 {
        self.number_of_nodes
    }
    pub fn build_graph_on_startup(&self) -> bool {
        self.build_graph_on_startup
    }
    pub fn geojson_export_path(&self) -> &Option<String> {
        &self.geojson_export_path
    }
    pub fn max_test(&self) -> bool {
        self.max_test
    }
}