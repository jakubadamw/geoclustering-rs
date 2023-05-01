#![allow(dead_code)]

use itertools::Itertools;
use structopt::StructOpt;
use strum::VariantNames;

#[derive(Debug, strum::EnumVariantNames, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Algorithm {
    Dbscan,
    Optics,
}

#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Options {
    #[structopt(short, long)]
    distance: f64,
    #[structopt(short, long)]
    size: usize,
    #[structopt(short, long, default_value = "./output")]
    output: std::path::PathBuf,
    #[structopt(short, long, possible_values = Algorithm::VARIANTS, default_value = "dbscan")]
    algorithm: Algorithm,
    #[structopt(long)]
    open: bool,
    #[structopt(long)]
    debug: bool,
    input: std::path::PathBuf,
}

fn distance_in_km_to_radians(distance_in_km: f64) -> f64 {
    distance_in_km / 6378.1
}

fn main() -> anyhow::Result<()> {
    use linfa::traits::Transformer;

    let options = Options::from_args();

    println!("Reading the CSV file from {}…", options.input.display());

    let mut reader = csv::Reader::from_path(options.input)?;

    let headers = reader.headers()?.clone();

    let get_float_value = |record: &csv::StringRecord, column_name: &str| -> Option<f64> {
        let index = headers.iter().position(|header| header == column_name)?;
        let degrees: f64 = record.get(index)?.parse().ok()?;
        Some(degrees.to_radians())
    };

    let locations = reader
        .into_records()
        .filter_map_ok(|record| {
            Some([
                get_float_value(&record, "lat")?,
                get_float_value(&record, "lon")?,
            ])
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("Found {} locations in the CSV file.", locations.len());

    let array = ndarray::arr2(&locations);

    let _clusters = linfa_clustering::Optics::params(options.size)
        .tolerance(distance_in_km_to_radians(options.distance))
        .transform(array.view())
        .unwrap();

    Ok(())
}
