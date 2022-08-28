mod influx;
mod kp_data;

/*
https://www.gfz-potsdam.de/en/kp-index
https://www-app3.gfz-potsdam.de/kp_index/Kp_ap_nowcast.txt
https://qubyte.codes/blog/parsing-input-from-stdin-to-structures-in-rust
*/
use std::{
    process,
    fs
};

use influx::*;
use kp_data::*;

use clap::Parser;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(
        short,
        long,
        value_parser,
        default_value = DEFAULT_URL_GFZ_KP_AP_NOWCAST
    )]
    url: String,

    #[clap(
        short,
        long,
        value_parser,
        default_value = ""
    )]
    cache_file: String,

    #[clap(
        short,
        long,
        action
    )]
    diagnostic_output: bool,
}

fn main() {
    let args = Args::parse();
    let url = args.url;
    let c_path = args.cache_file;
    //println!("Download now cast from '{0}' ...", url);

    let res = reqwest::blocking::get(&url).unwrap_or_else(|err| {
        eprintln!("Error while downloading '{url}': {err}");
        process::exit(1)
    });


    let con = res.text().unwrap_or_else(|err| {
        eprintln!("Error while downloading '{url}': {err}");
        process::exit(1)
    });

    let kp_file = con.parse::<KpFile>().unwrap_or_else(|err| {
        eprintln!("Error parsing file: {err}");
        process::exit(1)
    });

    if args.diagnostic_output {
        println!("Downloaded Kp file: {}", kp_file);
    }

    let kp_cache = if !c_path.is_empty() && fs::metadata(&c_path).is_ok() {
        let c = fs::read_to_string(&c_path).unwrap_or_else(|err| {
            eprintln!("Error reading file '{c_path}': {err}");
            process::exit(1)
        });

        c.parse::<KpFile>().unwrap_or_else(|err| {
            eprintln!("Error parsing cache file content: {err}");
            process::exit(1)
        })
    } else {
        KpFile::new()
    };

    let new_entries = kp_file.get_new_entries(&kp_cache);

    if args.diagnostic_output {
        println!("Cached Kp file: {}", kp_cache);

        for entry in new_entries {
            println!("{}", entry);
        }
    } else {
        for entry in new_entries {
            let m: Measurement = entry.into();
            println!("{}", m);
        }
    }

    if !c_path.is_empty() {
        let _ = fs::write(&c_path, &con).unwrap_or_else(|err| {
            eprintln!("Error writing file '{c_path}': {err}");
            process::exit(1)
        });
    }

}

impl From<&Entry> for Measurement {
    fn from(e: &Entry) -> Measurement {
        let mut m = Measurement::new("iono_activity");

        m.add_value("kp", e.kp.into(), false)
        .add_value("ap", e.ap.into(), false)
        .add_tag("def", &e.d.to_string(), false)
        .set_time( e.date );
        
        m
    }
}