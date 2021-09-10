use crate::slam3_orb::Slam3ORB;
use crate::ImageDb;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use opencv::{core, features2d, flann};
use std::str::FromStr;
use structopt::clap::AppSettings;
use structopt::StructOpt;

pub static OPTS: Lazy<Opts> = Lazy::new(Opts::from_args);
pub static DB_PATH: Lazy<String> = Lazy::new(|| {
    let proj_dirs = ProjectDirs::from("", "aloxaf", "imsearch").expect("failed to get project dir");
    proj_dirs
        .config_dir()
        .join("imsearch.db")
        .to_string_lossy()
        .to_string()
});

#[derive(StructOpt)]
#[structopt(name = "imsearch", global_setting(AppSettings::ColoredHelp))]
pub struct Opts {
    /// Path to image feature database
    #[structopt(short, long, default_value = &*DB_PATH)]
    pub db_path: String,
    /// The maximum number of features to retain
    #[structopt(short = "n", value_name = "N", long, default_value = "500")]
    pub orb_nfeatures: u32,
    /// Pyramid decimation ratio, greater than 1
    #[structopt(long, value_name = "SCALE", default_value = "1.2")]
    pub orb_scale_factor: f32,
    /// The number of pyramid levels
    #[structopt(long, value_name = "N", default_value = "8")]
    pub orb_nlevels: u32,
    /// Initial fast threshold
    #[structopt(long, value_name = "THRESHOLD", default_value = "20")]
    pub orb_ini_th_fast: u32,
    /// Minimum fast threshold
    #[structopt(long, value_name = "THRESHOLD", default_value = "7")]
    pub orb_min_th_fast: u32,
    /// The number of hash tables to use
    #[structopt(long, value_name = "NUMBER", default_value = "6")]
    pub flann_table_number: i32,
    /// The length of the key in the hash tables
    #[structopt(long, value_name = "SIZE", default_value = "12")]
    pub flann_key_size: i32,
    /// Number of levels to use in multi-probe (0 for standard LSH)
    #[structopt(long, value_name = "LEVEL", default_value = "1")]
    pub flann_probe_level: i32,
    #[structopt(long, value_name = "CHECKS", default_value = "32")]
    pub flann_checks: i32,
    #[structopt(long, value_name = "EPS", default_value = "0.0")]
    pub flann_eps: f32,
    /// Number of features to search per iteration
    #[structopt(long, value_name = "SIZE", default_value = "5000000")]
    pub batch_size: usize,
    /// How many results to show
    #[structopt(long, value_name = "COUNT", default_value = "10")]
    pub output_count: usize,
    /// Output format
    #[structopt(long, value_name = "FORMAT", default_value = "table", possible_values = &["table", "json"])]
    pub output_format: OutputFormat,
    /// Count of best matches found per each query descriptor
    #[structopt(long, value_name = "K", default_value = "3")]
    pub knn_k: i32,
    #[structopt(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(StructOpt)]
pub enum SubCommand {
    /// Show all features point for an image
    ShowKeypoints(ShowKeypoints),
    /// Show matches between two image
    ShowMatches(ShowMatches),
    /// Add images to database
    AddImages(AddImages),
    /// Search image from database
    SearchImage(SearchImage),
}

#[derive(StructOpt)]
pub struct ShowKeypoints {
    /// Path to an image
    pub image: String,
    /// Optional output image
    pub output: Option<String>,
}

#[derive(StructOpt)]
pub struct ShowMatches {
    /// Path to image A
    pub image1: String,
    /// Path to image B
    pub image2: String,
    /// Optional output image
    pub output: Option<String>,
}

#[derive(StructOpt)]
pub struct AddImages {
    /// Path to an image or folder
    pub path: String,
    /// Scan image with these suffixes
    #[structopt(short, long, default_value = "jpg,png")]
    pub suffix: String,
}

#[derive(StructOpt)]
pub struct SearchImage {
    /// Path to the image to search
    pub image: String,
}

#[derive(StructOpt)]
pub enum OutputFormat {
    Json,
    Table,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "table" => Ok(Self::Table),
            _ => unreachable!(),
        }
    }
}

impl From<&Opts> for Slam3ORB {
    fn from(opts: &Opts) -> Self {
        Self::create(
            opts.orb_nfeatures as i32,
            opts.orb_scale_factor,
            opts.orb_nlevels as i32,
            opts.orb_ini_th_fast as i32,
            opts.orb_min_th_fast as i32,
        )
        .expect("failed to build Slam3Orb")
    }
}

impl From<&Opts> for features2d::FlannBasedMatcher {
    fn from(opts: &Opts) -> Self {
        let index_params = core::Ptr::new(flann::IndexParams::from(
            flann::LshIndexParams::new(
                opts.flann_table_number,
                opts.flann_key_size,
                opts.flann_probe_level,
            )
            .expect("failed to build LshIndexParams"),
        ));
        let search_params = core::Ptr::new(
            flann::SearchParams::new_1(opts.flann_checks, opts.flann_eps, true)
                .expect("failed to build SearchParams"),
        );
        features2d::FlannBasedMatcher::new(&index_params, &search_params)
            .expect("failed to build FlannBasedMatcher")
    }
}

impl From<&Opts> for ImageDb {
    fn from(opts: &Opts) -> Self {
        Self::new(&opts.db_path).expect("failed to create ImageDb")
    }
}