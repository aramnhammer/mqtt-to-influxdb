//! A mqtt broker which listens on any topic, parses topics and messages into an adafruite compatible format, and pushes the result to influxdb2
//! A configuration file is necessary to invoke the mqtt broker library.
//! Environment variables are necessary to provide credentials and urls for the database server.
//! All configs can be found under `configs` directory
//!
//! # Dev environment
//! To set up a dev environment, follow the directions in the README.md
//!

// env
use environ;
use std::{fmt::Result, fs::metadata};

// influx;
use influx_client;

// messages
mod deserializer;

// MQTT SERVER
use librumqttd::{async_locallink::construct_broker, Config};
use std::thread;

// SERIALIZATION
extern crate serde_json;

// logging
use log::{debug, error, info, trace, warn, LevelFilter, SetLoggerError};

use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
    filter::{threshold::ThresholdFilter, Response},
};

fn build_logger() -> Result {
    let level = log::LevelFilter::Info;
    let file_path = "log/server.log";

    // Build a stderr logger.
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    let _handle = log4rs::init_config(config).unwrap();
    Ok(())
}

struct Settings {
    server_host: String,
    server_token: String,
    org: String,
    bucket: String,
}

impl Settings {
    /* loads expected environment variables */
    fn new() -> Settings {
        let db_token = "DB_SERVER_TOKEN";
        let db_host = "DB_SERVER_HOST";
        let db_org = "DB_ORG";
        let db_bucket = "DB_BUCKET";

        let server_token = std::env::var(db_token).unwrap();
        let server_host = std::env::var(db_host).unwrap();
        let org = std::env::var(db_org).unwrap();
        let bucket = std::env::var(db_bucket).unwrap();

        return Settings {
            server_token,
            server_host,
            org,
            bucket,
        };
    }
}

fn main() {
    build_logger().unwrap();
    let config_path = "config/server.env";
    match metadata(config_path) {
        Ok(_) => environ::EnvironmentLoader::new(config_path),
        Err(_) => info!("Using system environment"),
    }

    let settings = Settings::new();
    info!("Loaded settings");

    let config: Config = confy::load_path("config/rumqttd.conf").unwrap();
    info!("Loaded mqtt broker config");
    let (mut router, console, servers, builder) = construct_broker(config);
    info!("Constructed broker");

    thread::spawn(move || {
        router.start().unwrap();
        info!("Router thread started");
    });

    let mut rt = tokio::runtime::Builder::new_multi_thread();

    let server_host = settings.server_host;
    let org = settings.org;
    let bucket = settings.bucket;

    rt.enable_all();
    rt.build().unwrap().block_on(async {
        let (mut tx, mut rx) = builder.connect("localclient", 200).await.unwrap();
        tx.subscribe(std::iter::once("#")).await.unwrap();
        let console_task = tokio::spawn(console);
        info!("Server is listening");
        let sub_task = tokio::spawn(async move {
            loop {
                let message = rx.recv().await.unwrap();
                info!("T = {}, P = {:?}", message.topic, message.payload);
                if !message.payload.is_empty() {
                    let session = influx_client::InfluxSession::new(
                        server_host.clone(), // todo: change all these to refs
                        settings.server_token.clone(),
                        org.clone(),
                        bucket.clone(),
                        influx_client::Precision::S,
                    );

                    let point = deserializer::deserialize_message(&message.topic, &message.payload);
                    session.push_points(point).await;
                }
            }
        });

        servers.await;
        sub_task.await.unwrap();
        console_task.await.unwrap();
    });
}
