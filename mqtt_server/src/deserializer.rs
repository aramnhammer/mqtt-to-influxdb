use std::collections::HashMap;

use influx_client;
use log::{debug, error, info, trace, warn};
extern crate bytes;
extern crate serde_json;

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use bytes::Buf;

    use super::{deserialize_bytes_to_str, json_to_hashmap, topic_parser};

    #[test]
    fn topic_parses_adafruite_format() {
        let input_topic_str = "bucket/f/measurement.field_key".to_string();
        let res = topic_parser(&input_topic_str);
        assert_eq!(res.0, "measurement");
        assert_eq!(res.1, "field_key");
    }
    #[test]
    fn topic_parser_handles_no_field_key() {
        let input_topic_str = "bucket/f/measurement".to_string();
        let res = topic_parser(&input_topic_str);
        assert_eq!(res.0, "measurement");
        assert_eq!(res.1, "default");
    }
    #[test]
    fn topic_parser_handles_empty_topic_string() {
        let input_topic_str = "".to_string();
        let res = topic_parser(&input_topic_str);
        assert_eq!(res.0, "unknown_measurement");
        assert_eq!(res.1, "default");
    }
    #[test]
    fn topic_parser_handles_only_bucket_and_f() {
        let input_topic_str = "bucket/f/".to_string();
        let res = topic_parser(&input_topic_str);
        assert_eq!(res.0, "unknown_measurement");
        assert_eq!(res.1, "default");
    }
    #[test]
    fn deserialize_bytes_results_in_lower_case() {
        let expected_result = "{},234 Hello World".to_lowercase();
        let bytes = (&b"{},234 Hello World"[..]).copy_to_bytes(18);
        let mut v: Vec<bytes::Bytes> = Vec::new();
        v.push(bytes);
        let res = deserialize_bytes_to_str(&v);
        assert_eq!(res, expected_result);
    }
    #[test]
    fn json_to_hashmap_returns_defaults_on_failure_string_value() {
        let bad_field = "{\"value\": ''}".to_string();
        let mut expect = HashMap::new();
        expect.insert("value".to_string(), 0.0);
        let res = json_to_hashmap(&bad_field);
        assert_eq!(res, expect);
    }
}

pub fn topic_parser(topic: &String) -> (String, String) {
    // default measurement and field_key
    let mut measurement = "unknown_measurement".to_string();
    let mut field_key = "default".to_string();

    let split_topic_str: Vec<&str> = topic.split("/").collect(); // bucket/f/measurement.field_key

    if split_topic_str.len() <= 2 {
        return (measurement.to_string(), field_key.to_string());
    }

    let split_measurement_field: Vec<&str> = split_topic_str[2].split(".").collect();

    // no field key found
    if split_measurement_field.len() == 1 {
        if split_measurement_field[0].to_string() != "" {
            measurement = split_measurement_field[0].to_string();
        }
        field_key = "default".to_string();

    // both field and measurement found
    } else if split_measurement_field.len() == 2 {
        measurement = split_measurement_field[0].to_string();
        field_key = split_measurement_field[1].to_string();
    }

    info!(
        "Measurement: {measurement} - field: {field}",
        measurement = measurement,
        field = field_key
    );
    return (measurement.to_string(), field_key.to_string());
}

fn deserialize_bytes_to_str(payload: &Vec<bytes::Bytes>) -> String {
    let mut buf = String::new();
    for b in payload.into_iter() {
        let a = String::from_utf8(b.to_vec());
        buf.push_str(&a.unwrap().to_lowercase());
    }
    return buf;
}

fn json_to_hashmap(buf: &String) -> HashMap<String, f64> {
    let mut default_field = HashMap::new();
    default_field.insert("value".to_string(), 0.0);
    let fields: HashMap<String, f64> = serde_json::from_str(&buf).unwrap_or(default_field.clone());
    if fields == default_field {
        warn!(
            "WARNING: failed to serialize fields to json form, using default fields: bad payload => {buf}",
            buf = buf,
        )
    }
    if !fields.contains_key("value") {
        // only time we panic
        panic!(
            "WARNING: payload does not contain 'value' key => {buf}",
            buf = buf,
        )
    }
    return fields;
}

pub fn deserialize_message(topic: &String, payload: &Vec<bytes::Bytes>) -> influx_client::Point {
    let measurement_and_field = topic_parser(topic);
    let buf = deserialize_bytes_to_str(payload);
    info!("Unpacked payload: {payload}", payload = buf);
    let fields = json_to_hashmap(&buf);
    let mut adafruite_formatted_message: HashMap<String, f64> = HashMap::new();
    let value = fields.get("value").unwrap().to_owned(); // should always be lowercase from the deserialize_bytes_to_str method
    adafruite_formatted_message.insert(measurement_and_field.1, value);
    return influx_client::Point {
        measurement: measurement_and_field.0,
        field_map: adafruite_formatted_message,
    };
}
