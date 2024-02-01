use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diqwest::WithDigestAuth;
use influxdb::{InfluxDbWriteable, WriteQuery};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::{Number, Value};

use crate::util::{electricity_rate, joules_to_watts};

#[derive(Clone, Copy, Deserialize, Debug, InfluxDbWriteable)]
#[allow(dead_code)]
pub struct PerMinuteZappiData {
    pub time: DateTime<Utc>, // parsed from yr mon dom hr
    pub imported_watts: i64, // from imp value
    pub exported_watts: i64, // from exp value
    pub generated_watts: i64, // from gep value (generated positive). (gen is perhaps the standby power of the solar inverter)
    pub zappi_watts: i64, // h1b
    pub electricity_rate: f32,
}

impl PerMinuteZappiData {
    pub fn new(data: Value) -> PerMinuteZappiData {
        let year = data.get("yr").unwrap().to_string().replace("\"", "");
        let month = data.get("mon").unwrap().to_string().replace("\"", "");
        let dom = data.get("dom").unwrap().to_string().replace("\"", "");
        let hour = data.get("hr").unwrap_or(&Value::String(String::from("0"))).to_string().replace("\"", "");
        let minute = data.get("min").unwrap_or(&Value::String(String::from("0"))).to_string().replace("\"", "");
        let parsed_datetime = format!("{}-{}-{} {}:{}:00 +0000", year, month, dom, hour, minute);
        let time: DateTime::<Utc> = DateTime::parse_from_str(&parsed_datetime, "%Y-%m-%d %H:%M:%S %z").unwrap().into();
        let imported_joules = data.get("imp").unwrap_or(&Value::Number(Number::from(0))).as_i64().unwrap();
        let exported_joules = data.get("exp").unwrap_or(&Value::Number(Number::from(0))).as_i64().unwrap();
        let generated_joules = data.get("gep").unwrap_or(&Value::Number(Number::from(0))).as_i64().unwrap();
        let zappi_joules = data.get("h1b").unwrap_or(&Value::Number(Number::from(0))).as_i64().unwrap();

        PerMinuteZappiData {
            time,
            imported_watts: joules_to_watts(imported_joules, 60),
            exported_watts: joules_to_watts(exported_joules, 60),
            generated_watts: joules_to_watts(generated_joules, 60),
            zappi_watts: joules_to_watts(zappi_joules, 60),
            electricity_rate: electricity_rate(time.time()),
        }
    }
}

pub async fn get_day_of_zappi_data(asn: &str, serial_no: &str, api_key: &str, date: &str) -> Result<Vec::<PerMinuteZappiData>, Box<dyn std::error::Error>> {
    let url = format!("https://{}/cgi-jday-Z{}-{}", asn, serial_no, date);
    println!("Fetching hourly Zappi data from {}", url);
    let data_resp: Response = Client::new()
      .get(&url)
      .send_with_digest_auth(serial_no, api_key).await?;

    let data: HashMap<String,Vec<Value>> = data_resp.json().await?;
    println!("{:#?}", data);

    let collection = data.get(&format!("U{}", serial_no)).unwrap();
    let mut day_data = Vec::<PerMinuteZappiData>::new();

    for hour in collection.iter() {
        day_data.push(PerMinuteZappiData::new(hour.clone()))
    }

    Ok(day_data)
}

pub async fn write_daily_zappi_data(influx_client: &influxdb::Client, data: &Vec::<PerMinuteZappiData>) -> 
    Result<std::string::String, influxdb::Error> {
    let mut zappi_readings = Vec::<WriteQuery>::new();

    for minute in data.iter() {
        zappi_readings.push(minute.into_query("zappi_data"));
    }

    influx_client.query(zappi_readings).await
}
