#![allow(unused_variables)]
extern crate chrono;
extern crate csv;
#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate log;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate simplelog;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate toml;

use chrono::offset::Utc;
use csv::WriterBuilder;
use std::fs::File;

mod data;
use data::{ActualConfig, Alltime, Error, User};
use structopt::StructOpt;

const BASE_URL: &str = "https://api.streamelements.com/kappa/v2";

type Result<T> = std::result::Result<T, Error>;

fn main() {
    match run() {
        Ok(_) => println!("Program completed without errors"),
        Err(e) => {
            eprintln!("Program errored: {}", e);
            error!("Error: {}", e)
        }
    }
}

fn run() -> Result<()> {
    let matches = Opts::from_args();
    let config = load_toml()?;
    let channel = config.channel();
    info!("loaded channel: {}", channel);
    let request = reqwest::Client::new();
    trace!("created reqwest client");

    let alltime_url = format!("{}/points/{}/alltime?limit=1000", BASE_URL, channel);
    let top_url = format!("{}/points/{}/top?limit=1000", BASE_URL, channel);

    let (url, opt) = match matches {
        Opts::Alltime => (alltime_url, "alltime"),
        Opts::Top => (top_url, "top"),
    };
    let response: Alltime = request.get(&url).send()?.json()?;
    info!("received response from streamelements api");

    let today = Utc::today().format("%d-%m-%Y");
    info!("date for filename: {}", today);

    let mut csv = WriterBuilder::new()
        .has_headers(false)
        .from_path(format!("{}-{}-points.csv", today, opt))?;
    info!("successfully created csv writer");
    if let Some(cutoff) = config.cutoff() {
        let last_point = response
            .users()
            .last()
            .expect("No users were returned from the api")
            .points;
        if last_point < cutoff {
            let filtered: Vec<User> = response
                .into_users()
                .into_iter()
                .filter(|user| user.points > cutoff)
                .collect();
            write_to_csv(&mut csv, filtered.as_slice())?;
            return Ok(());
        }
    }
    write_to_csv(&mut csv, &response.users())?;

    // We request 1000 initially, if the total is less
    // then we found them all
    if response._total < 1000 {
        return Ok(());
    }
    // Rounded up integer division, rust rounds towards
    // zero as that is what llvm does.
    // We are requesting the max number of records, 1000
    let offset_count = ((response._total - 1) / 1000) + 1;
    for offset in 2..offset_count + 1 {
        let offset = offset * 1000;
        let resp: Alltime = request
            .get(&format!("{}&offset={}", url, offset))
            .send()?
            .json()?;
        info!("received response from streamelements api");

        // Not sure how to build around this without duplicating code
        // looks very messy
        if let Some(cutoff) = config.cutoff() {
            let last_point = resp.users()
                .last()
                .expect("No users were returned from the api")
                .points;
            if last_point < cutoff {
                let filtered: Vec<User> = resp.into_users()
                    .into_iter()
                    .filter(|user| user.points > cutoff)
                    .collect();
                write_to_csv(&mut csv, filtered.as_slice())?;
                break;
            }
        }
        write_to_csv(&mut csv, &resp.users())?;
        info!("successfully wrote to csv");
    }
    Ok(())
}

/// Convenience function because Toml doesn't support reading from a file
/// serde_json does, :/
fn load_toml() -> Result<ActualConfig> {
    use std::io::Read;
    let mut file = File::open("./config.toml")?;
    let mut str_bufr = String::new();
    file.read_to_string(&mut str_bufr)?;
    Ok(toml::from_str(&str_bufr)?)
}

/// Convenience function to serialize to a CSV via an iterator.
/// TODO: Generics
fn write_to_csv(csv: &mut csv::Writer<File>, users: &[User]) -> Result<()> {
    for user in users {
        csv.serialize(user)?;
    }
    Ok(())
}


#[derive(Debug, StructOpt)]
#[structopt(name = "streamelements-csv")]
enum Opts {
    #[structopt(name = "alltime")]
    /// Uses the api to get the alltime top users
    Alltime,
    #[structopt(name = "top")]
    /// Uses the api to get the Top users currently
    Top,
}
