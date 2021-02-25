#[macro_use]
extern crate clap;

use std::fs::{create_dir_all, read_to_string, write};

use chrono::prelude::*;
use clap::{App, Arg};
use directories::ProjectDirs;

fn get_rate(
    from: &str,
    to: &str,
    crypto_list: String,
    fiat_list: String,
) -> Result<f64, reqwest::Error> {
    let fiat_json: serde_json::Value =
        serde_json::from_str(&fiat_list).expect("The result doesn't seem to be JSON");
    let crypto_json: serde_json::Value =
        serde_json::from_str(&crypto_list).expect("The result doesn't seem to be JSON");

    let fiat_object = fiat_json["rates"].as_object().unwrap();
    let crypto_array = crypto_json["data"]["coins"].as_array().unwrap();

    let from_val = if fiat_object.contains_key(from) {
        1.0 / fiat_object[from].as_f64().unwrap()
    } else if crypto_array
        .iter()
        .any(|x| x.as_object().unwrap()["symbol"] == from.to_owned())
    {
        let mut c = crypto_array[crypto_array
            .iter()
            .position(|x| x.as_object().unwrap()["symbol"] == from.to_owned())
            .unwrap()]["price"]
            .to_string();

        c.pop();
        c[1..c.len()].parse::<f64>().unwrap()
    } else {
        panic!(
            "The currency symbol \"{}\" is incorrect or not available.",
            from
        )
    };

    let to_val = if fiat_object.contains_key(to) {
        1.0 / fiat_object[to].as_f64().unwrap()
    } else if crypto_array
        .iter()
        .any(|x| x.as_object().unwrap()["symbol"] == to.to_owned())
    {
        let mut c = crypto_array[crypto_array
            .iter()
            .position(|x| x.as_object().unwrap()["symbol"] == to.to_owned())
            .unwrap()]["price"]
            .to_string();

        c.pop();
        c[1..c.len()].parse::<f64>().unwrap()
    } else {
        panic!(
            "The currency symbol \"{}\" is incorrect or not available.",
            to
        )
    };

    Ok(from_val / to_val)
}

fn read_cache(path: &String) -> Result<String, std::io::Error> {
    match read_to_string(path) {
        Ok(str) => Ok(str),
        Err(why) => panic!("An error occured while reading the cache: {}", why),
    }
}

fn cache_data(path: &String, data: &String) {
    match write(path, data) {
        Ok(_) => (),
        Err(why) => panic!("An error occured during caching: {}", why),
    }
}

fn fetch_fiat_list() -> Result<String, reqwest::Error> {
    let url = "https://api.ratesapi.io/api/latest?base=USD";
    let body = reqwest::blocking::get(url)?.text()?;

    Ok(body)
}

fn fetch_crypto_list() -> Result<String, reqwest::Error> {
    let url = "https://api.coinranking.com/v2/coins";
    let body = reqwest::blocking::get(url)?.text()?;

    Ok(body)
}

fn init_currency_data() -> Result<(String, String), std::io::Error> {
    // Define paths to cached files
    let proj_dirs = ProjectDirs::from("rs", "Lunush", "Rates").unwrap();
    let cache_dir = proj_dirs.cache_dir().to_str().unwrap().to_owned();
    let crypto_list_path = format!(
        "{}/crypto_list.json",
        proj_dirs.cache_dir().to_str().unwrap()
    )[..]
        .to_owned();
    let fiat_list_path =
        format!("{}/fiat_list.json", proj_dirs.cache_dir().to_str().unwrap())[..].to_owned();
    let last_update_path =
        format!("{}/last_update", proj_dirs.cache_dir().to_str().unwrap())[..].to_owned();

    // Create cache directory if it doesn't exist
    match create_dir_all(&cache_dir) {
        Err(why) => panic!("Unable to create {} folder:\n\n{}", &cache_dir, why),
        Ok(_) => (),
    };

    // If last_update file does not exist or was updated >3 hours ago, pull the data. Otherwise use
    // cache.
    let crypto_list: String;
    let fiat_list: String;

    let now = Utc::now().timestamp();
    match read_to_string(&last_update_path) {
        Ok(time) => {
            let last_update_time = time.parse::<i64>().unwrap();
            const HOUR: i64 = 3600;

            if last_update_time + HOUR * 3 < now {
                crypto_list = fetch_crypto_list().unwrap();
                fiat_list = fetch_fiat_list().unwrap();

                cache_data(&crypto_list_path, &crypto_list);
                cache_data(&fiat_list_path, &fiat_list);
                cache_data(&last_update_path, &now.to_string());
            } else {
                crypto_list = read_cache(&crypto_list_path).unwrap();
                fiat_list = read_cache(&fiat_list_path).unwrap();
            }
        }
        Err(_) => {
            crypto_list = fetch_crypto_list().unwrap();
            fiat_list = fetch_fiat_list().unwrap();

            cache_data(&crypto_list_path, &crypto_list);
            cache_data(&fiat_list_path, &fiat_list);
            cache_data(&last_update_path, &now.to_string());
        }
    };

    Ok((crypto_list.to_owned(), fiat_list.to_owned()))
}

fn main() -> Result<(), std::io::Error> {
    // Initialize CLI
    let cli = App::new("Rates")
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::with_name("from")
                .help("Currency you want to convert from")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("to")
                .help("Currency you want to convert to")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("amount")
                .short("a")
                .long("amount")
                .takes_value(true)
                .help("Amount of a given currency. Defaults to 1"),
        )
        .arg(
            Arg::with_name("short")
                .short("s")
                .long("short")
                .help("Show only the result value"),
        )
        .get_matches();

    // Define values
    let amount = if cli.is_present("amount") == true {
        cli.value_of("amount").unwrap().parse::<f64>().unwrap()
    } else {
        1.0
    };
    let from = String::from(cli.value_of("from").unwrap().to_uppercase());
    let to = String::from(cli.value_of("to").unwrap().to_uppercase());
    let short = cli.is_present("short");
    let (crypto_list, fiat_list) = init_currency_data()?;

    let to_val = get_rate(&from, &to, crypto_list, fiat_list).unwrap() * &amount;

    if short == true {
        println!("{}", to_val);
    } else {
        println!("{} {} = {} {}", amount, from, to_val, to);
    }

    Ok(())
}
