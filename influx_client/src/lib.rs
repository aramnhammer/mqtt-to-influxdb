//! A simple influxdb data pusher

extern crate reqwest;

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use log::{debug, error, info, trace, warn};
use reqwest::header;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// a point of data, where filed map is a <String, f64> mapping of influxdb2 field values, and measurement is the desired measurement string.
pub struct Point {
    pub measurement: String,
    pub field_map: HashMap<String, f64>,
}

pub struct InfluxSession {
    url: String,
    token: String,
    org: String,
    bucket: String,
    precision: String,
}
pub enum Precision {
    S, // Seconds
}

impl Precision {
    pub fn value(&self) -> &str {
        match *self {
            Precision::S => "s",
        }
    }
}

async fn make_request(
    url: &String,
    token: &String,
    org: &String,
    bucket: &String,
    precision: &String,
    measurement: &String,
    fields: &HashMap<String, f64>,
    timestamp: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    /*
    Whitespace in line protocol determines how InfluxDB interprets the data point.
    The first unescaped space delimits the measurement and the tag set from the field set.
    The second unescaped space delimits the field set from the timestamp.
    */
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Token {token}", token = token).parse().unwrap(),
    );
    let mut body = String::new();
    for (field_key, field_value) in fields {
        if body.is_empty() {
            body.push_str(&measurement); // first string is measurement
            body.push_str(" "); // first space delim field set from tag set
        } else {
            body.push_str(","); // comma delim fields
        }
        let field = format!(
            "{field_key}={field_value}",
            field_key = field_key,
            field_value = field_value
        );
        body.push_str(&field);
    }
    body.push_str(&format!(" {timestamp}", timestamp = timestamp)); // last space delims timestamp from everything else
    info!("{}", body);

    let res = reqwest::Client::new()
        .post(format!(
            "{url}/api/v2/write?org={org}&bucket={bucket}&precision={precision}",
            url = url,
            org = org,
            bucket = bucket,
            precision = precision
        ))
        .headers(headers)
        .body(body)
        .send()
        .await?
        .text()
        .await?;
    info!("{}", res);
    Ok(())
}

impl InfluxSession {
    pub fn new<'a>(
        url: String,
        token: String,
        org: String,
        bucket: String,
        precision: Precision,
    ) -> InfluxSession {
        let precision = precision.value().to_string();
        return InfluxSession {
            url,
            token,
            org,
            bucket,
            precision,
        };
    }
    pub async fn push_points(self: &Self, point: Point) {
        let milis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_millis();
        let sec = (milis / 1000) as i64;
        let nanos = ((milis % 1000) * 1_000_000) as u32;
        let ts = format!(
            "{ts}",
            ts = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(sec, nanos), Utc)
                .timestamp()
        );

        match make_request(
            &self.url,
            &self.token,
            &self.org,
            &self.bucket,
            &self.precision,
            &point.measurement,
            &point.field_map,
            &ts,
        )
        .await
        {
            Err(e) => info!("FAILED TO PUSH: {:?}", e),
            _ => (),
        }
    }
}
