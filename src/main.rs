use clap::{Arg, App};

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
let cli = App::new("Coin")
                          .version("0.1.0")
                          .about("Get latest exchange rates")
                          .arg(Arg::with_name("FROM")
                               .help("Currency you want to convert from")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("TO")
                               .help("Currency you want to convert to")
                               .required(true)
                               .index(2))
                          .get_matches();

    let base = String::from(cli.value_of("FROM").unwrap().to_uppercase());
    let to = String::from(cli.value_of("TO").unwrap().to_uppercase());
    let url = &format!("https://api.exchangeratesapi.io/latest?base={}", &base)[..];
    let res = reqwest::get(url).await?;
    println!("Status: {}", res.status());

    let body = res.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body).expect("The result doesn't seem to be JSON");
    let to_val = json["rates"][&to].to_string().parse::<f32>().unwrap();
    println!("1 {} = {:?} {}", base, to_val, to);

    Ok(())
}
