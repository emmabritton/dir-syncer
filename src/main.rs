use clap::{App, Arg, Values, crate_name, crate_version, crate_authors, crate_description};
use crate::file_checker::FileChecker;
use regex::Regex;
use crate::syncer::Syncer;
use simplelog::{SimpleLogger, ConfigBuilder};
use log::{info, error, LevelFilter};
use std::time::{Duration, Instant};
use std::ops::Add;
use chrono::prelude::*;

pub type Error = Box<dyn std::error::Error>;

const FORMAT_NEXT_CHECK: &'static str = "%H:%M:%S";

mod file_checker;
mod syncer;

fn main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("src_dir")
            .value_name("DIR")
            .takes_value(true)
            .required(true)
            .long("source_dir")
            .short("s")
            .help("The directory to sync actions from")
            .multiple(false)
            .number_of_values(1))
        .arg(Arg::with_name("dest_dir")
            .value_name("DIR")
            .takes_value(true)
            .required(true)
            .long("dest_dir")
            .short("d")
            .help("The directory to sync actions to")
            .multiple(false)
            .number_of_values(1))
        .arg(Arg::with_name("freq")
            .value_name("MINUTES")
            .takes_value(true)
            .required(false)
            .default_value("5")
            .validator(|value| value.parse::<usize>().map(|_| ()).map_err(|_| String::from("Must be positive whole number")))
            .long("freq")
            .short("f")
            .help("How frequently should this program check the directories")
            .long_help("How frequently (in minutes) this program check attempt to sync the directories, the timer is started once the last operation from the previous check completes.")
            .multiple(false)
            .number_of_values(1))
        .arg(Arg::with_name("operations")
            .value_name("NUMBER")
            .takes_value(true)
            .required(false)
            .default_value("1")
            .long("operations")
            .short("o")
            .validator(|value| value.parse::<usize>().map(|_| ()).map_err(|_| String::from("Must be positive whole number")))
            .help("How many operations should this program perform per check")
            .long_help("If set to 1 then this program will only add, modify or delete one file per check")
            .multiple(false)
            .number_of_values(1))
        .arg(Arg::with_name("include")
            .value_name("REGEX")
            .takes_value(true)
            .multiple(true)
            .short("i")
            .default_value(".*")
            .long("include")
            .required(false)
            .help("Regex for file name (including extension) to be sync'd"))
        .arg(Arg::with_name("exclude")
            .value_name("REGEX")
            .takes_value(true)
            .multiple(true)
            .short("e")
            .long("exclude")
            .required(false)
            .help("Regex for file name (including extension) to ignored when sync'ing"))
        .arg(Arg::with_name("check")
            .takes_value(false)
            .short("c")
            .long("check")
            .required(false)
            .help("Run program in check mode")
            .long_help("If set then this program will just print out the operations that it would perform and exits"))
        .arg(Arg::with_name("verbose")
            .takes_value(false)
            .short("v")
            .long("verbose")
            .help("Set verbosity of program (between 0 and 3)")
            .required(false)
            .multiple(true))
        .get_matches();

    let src_dir = matches.value_of("src_dir").expect("[CLAP ERROR] No source dir").to_string();
    let dest_dir = matches.value_of("dest_dir").expect("[CLAP ERROR] No dest dir").to_string();
    let freq: u64 = matches.value_of("freq").expect("[CLAP ERROR] No frequency").parse().unwrap();
    let operations = matches.value_of("operations").expect("[CLAP ERROR] No operation count").parse().unwrap();
    let includes = matches.values_of("include").expect("[CLAP ERROR] No include pattern(s)").map(|text| Regex::new(text).unwrap()).collect();
    let excludes = matches.values_of("exclude").unwrap_or(Values::default()).map(|text| Regex::new(text).unwrap()).collect();
    let check = matches.is_present("check");
    let verbosity = matches.occurrences_of("verbose");

    let log_level = int_to_log_level(verbosity);

    let config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Trace)
        .build();

    SimpleLogger::init(log_level, config)?;

    let file_checker = FileChecker::new(
        src_dir.clone(),
        dest_dir.clone(),
        includes,
        excludes,
    );

    if check {
        match file_checker.get_list_of_files() {
            Ok(results) => {
                println!("{}", results);
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    } else {
        let syncer = Syncer::new(src_dir, dest_dir, operations);
        loop_and_sync(file_checker, syncer, Duration::from_secs(freq * 60));
    }

    Ok(())
}

fn loop_and_sync(file_checker: FileChecker, mut syncer: Syncer, sleep_time: Duration) {
    let chrono_sleep_time = chrono::Duration::from_std(sleep_time).expect("Frequency is too high");
    info!("Monitoring");
    loop {
        match file_checker.get_list_of_files() {
            Ok(results) => {
                if results.has_any_operations() {
                    syncer.set_results(results);
                    syncer.run();
                } else {
                    info!("Directories already sync'd");
                }
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
        let next = chrono::Local::now().add(chrono_sleep_time);
        info!("Next check at {}", next.format(FORMAT_NEXT_CHECK));
        std::thread::sleep(sleep_time);
    }
}

fn int_to_log_level(count: u64) -> log::LevelFilter {
    return match count.min(3) {
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Error
    };
}