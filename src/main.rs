#[macro_use]
extern crate clap;

use clap::{Arg, App};

async fn get(from: &str, to: &str) -> Result<String, reqwest::Error> {
    let url = &format!("https://api.exchangeratesapi.io/latest?base={}", &from)[..];
    let res = reqwest::get(url).await?;
    let body = res.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body).expect("The result doesn't seem to be JSON");
    let to_val = json["rates"][&to].to_string();

    Ok(to_val)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
let cli = App::new("Rates")
                          .version(crate_version!())
                          .about(crate_description!())
                          .author(crate_authors!())
                          .arg(Arg::with_name("from")
                               .help("Currency you want to convert from")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("to")
                               .help("Currency you want to convert to")
                               .required(true)
                               .index(2))
                          .arg(Arg::with_name("amount")
                               .short("a")
                               .long("amount")
                               .takes_value(true)
                               .help("Amount of a given currency. Defaults to 1"))
                          .arg(Arg::with_name("short")
                               .short("s")
                               .long("short")
                               .help("Show only the result value"))
                          .get_matches();

    let amount = cli.value_of("amount").unwrap().parse::<f64>().unwrap();
    let from = String::from(cli.value_of("from").unwrap().to_uppercase());
    let to = String::from(cli.value_of("to").unwrap().to_uppercase());
    let short = cli.is_present("short");

    let to_val = get(&from, &to).await.unwrap().parse::<f64>().unwrap() * amount;

    if short == true {
        println!("{}", to_val);
    } else {
        println!("{} {} = {} {}", amount, from, to_val, to);
    }

    Ok(())
}
