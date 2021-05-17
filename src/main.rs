use std::fs::{create_dir_all, read_to_string, write};

use chrono::prelude::*;
use directories::ProjectDirs;
use structopt::StructOpt;

fn get_rate(from: &str, to: &str, crypto_list: String, fiat_list: String) -> f64 {
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
        .any(|x| x.as_object().unwrap()["symbol"] == *from)
    {
        let mut c = crypto_array[crypto_array
            .iter()
            .position(|x| x.as_object().unwrap()["symbol"] == *from)
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
        .any(|x| x.as_object().unwrap()["symbol"] == *to)
    {
        let mut c = crypto_array[crypto_array
            .iter()
            .position(|x| x.as_object().unwrap()["symbol"] == *to)
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

    from_val / to_val
}

fn read_cache(path: &str) -> Result<String, std::io::Error> {
    match read_to_string(path) {
        Ok(str) => Ok(str),
        Err(why) => panic!("An error occured while reading the cache: {}", why),
    }
}

fn cache_data(path: &str, data: &str) {
    match write(path, data) {
        Ok(_) => (),
        Err(why) => panic!("An error occured during caching: {}", why),
    }
}

fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::blocking::get(url)?.text()?;

    Ok(body)
}

fn init_currency_data() -> (String, String) {
    // Define paths to cached files
    let proj_dirs = ProjectDirs::from("rs", "Lunush", "Rates").unwrap();
    let cache_dir = proj_dirs.cache_dir().to_str().unwrap().to_owned();
    let crypto_list_path = format!("{}/crypto_list.json", cache_dir)[..].to_owned();
    let fiat_list_path = format!("{}/fiat_list.json", cache_dir)[..].to_owned();
    let last_update_path = format!("{}/last_update", cache_dir)[..].to_owned();

    // Create cache directory if it doesn't exist
    if let Err(why) = create_dir_all(&cache_dir) {
        panic!("Unable to create {} folder:\n\n{}", cache_dir, why);
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
                crypto_list = fetch_data("https://api.coinranking.com/v2/coins").unwrap();
                fiat_list = fetch_data("https://api.ratesapi.io/api/latest?base=USD").unwrap();

                cache_data(&crypto_list_path, &crypto_list);
                cache_data(&fiat_list_path, &fiat_list);
                cache_data(&last_update_path, &now.to_string());
            } else {
                crypto_list = read_cache(&crypto_list_path).unwrap();
                fiat_list = read_cache(&fiat_list_path).unwrap();
            }
        }
        Err(_) => {
            crypto_list = fetch_data("https://api.coinranking.com/v2/coins").unwrap();
            fiat_list = fetch_data("https://api.ratesapi.io/api/latest?base=USD").unwrap();

            cache_data(&crypto_list_path, &crypto_list);
            cache_data(&fiat_list_path, &fiat_list);
            cache_data(&last_update_path, &now.to_string());
        }
    };

    (crypto_list, fiat_list)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rates", about = "Get financial data in your terminal")]
struct Args {
    /// (Optional) Amount of currency to convert from
    arg1: Option<String>,

    /// Currency to convert from
    arg2: Option<String>,

    /// (Optional) Currency to convert to. Defaults to USD if not provided
    arg3: Option<String>,

    /// Show only the result
    #[structopt(short = "s", long = "short")]
    short: bool,

    /// Trim the digits after decimal point, if any
    #[structopt(short = "t", long = "trim")]
    trim: bool,

    /// Do not format the result
    #[structopt(short = "F", long = "no-formatting")]
    no_formatting: bool,
}

#[derive(Debug)]
struct ParsedArgs {
    from: String,
    to: String,
    amount: f64,
}

fn parse_args(args: &Args) -> ParsedArgs {
    let arg1 = args.arg1.clone();
    let arg2 = args.arg2.clone();
    let arg3 = args.arg3.clone();

    if arg1.is_none() {
        panic!("At least one argument to convert from is expected");
    }

    let is_arg1_num = arg1.clone().unwrap().parse::<f64>().is_ok();

    let amount: f64;
    let from: String;
    let to: String;

    if is_arg1_num {
        amount = arg1.unwrap().parse::<f64>().unwrap();

        from = arg2.unwrap().to_uppercase();

        to = if let Some(arg) = arg3 {
            arg.to_uppercase()
        } else {
            "USD".to_owned()
        };
    } else {
        amount = 1.0;

        from = if let Some(arg) = arg1 {
            arg.to_uppercase()
        } else {
            panic!("At least one argument to convert from is expected");
        };

        to = if let Some(arg) = arg2 {
            arg.to_uppercase()
        } else {
            "USD".to_owned()
        };
    }

    ParsedArgs { from, to, amount }
}

fn main() {
    let args = Args::from_args();
    let ParsedArgs { from, to, amount } = parse_args(&args);

    // Define values
    let short = args.short;
    let trim = args.trim;
    let no_formatting = args.no_formatting;
    let (crypto_list, fiat_list) = init_currency_data();

    let mut to_val = amount * get_rate(&from, &to, crypto_list, fiat_list);

    // If trim set to true, trim all decimals. Show some decimals otherwise.
    if trim {
        to_val = to_val.floor();
    } else if !no_formatting {
        // 2 decimals if to_val > 1
        // 3 decimals if to_val > .1
        // 4 decimals if to_val > .01
        // etc
        let digits = to_val.to_string().chars().collect::<Vec<_>>();
        let mut decimal_length = 3;

        // Find the decimal point index
        let decimal_point_index = digits.iter().position(|x| *x == '.').unwrap();

        // If to_val < 1, search for the first 0 and when found trim the rest - 2.
        if to_val < 1.0 && digits[decimal_point_index + 1].to_digit(10).unwrap() == 0 {
            /* for digit in decimal_point_index + 1..digits.len() {
                if digits[digit] != '0' {
                    break;
                }
                decimal_length += 1;
            } */
            for digit in digits.iter().skip(decimal_point_index + 1) {
                if *digit != '0' {
                    break;
                }
                decimal_length += 1;
            }
        }

        to_val = digits[0..decimal_point_index + decimal_length]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .unwrap();
    }

    if short {
        println!("{}", to_val);
    } else {
        println!("{} {} = {} {}", amount, from, to_val, to);
    };
}
