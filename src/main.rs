use std::env;

use chrono::{Duration, Utc};
use diqwest::WithDigestAuth;
use reqwest::{Client, Response};
use zappi::write_daily_zappi_data;

use crate::zappi::get_day_of_zappi_data;

mod zappi;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let serial_no = env::var("SERIAL_NO").expect("$SERIAL_NO is not set");
    let api_key = env::var("API_KEY").expect("$API_KEY is not set");
    let influx_db_url = env::var("INFLUXDB_URL").expect("$INFLUXDB_URL is not set");
    let influx_client = influxdb::Client::new(&influx_db_url, "zappi");

    let asn_resp: Response = Client::new()
      .get("https://director.myenergi.net")
      .send_with_digest_auth(&serial_no, &api_key).await?;
    let asn = asn_resp.headers().get("x_myenergi-asn").unwrap().to_str().unwrap();

    let yesterday = (Utc::now() - Duration::days(1)).format("%Y-%-m-%-d").to_string();
    let daily_zappi_data = get_day_of_zappi_data(asn, &serial_no, &api_key, &yesterday).await.unwrap();

    write_daily_zappi_data(&influx_client, &daily_zappi_data).await.unwrap();

    Ok(())
}
