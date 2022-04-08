# mqtt-to-influxdb
An MQTT to Influxdb2 pusher designed to understand the ADAFRUIT mqtt topic format for time series data.

NOTE: `bootstrap.sh` will reset any database currently configured on a system. 

# Production
Potential production deployment method (No SSL/TLS)

- Add a persistant volume to the influxdb service under `tools/docker-compose.yaml`
- Edit the `bootstrap.sh` script changing the following values to your desired ones:
```
USERNAME="tester"
PASSWORD="Yegiazaryan"
ORG="test"
BUCKET="tests"
```
- Run `bootstrap.sh`

# Development Environment
How to test the server locally.

## Requirements
- rust
- docker (docker-compose)
- mqtt client, this one is good: http://mqtt-explorer.com/
- lib ssl:
  - pkg-config :  `sudo apt-get install pkg-config`  
  - lib ssl dev : `sudo apt install libssl-dev`

## Bringing up a local test environment
```
cd tools
./bootstrap.sh
```
- this will bring up (or reset) a new `influxdb2` instance in docker and generate a `server.env` file under the `config` directory
  
- running the `mqtt_server` (`cargo run mqtt_server`) will automatically load the `server.env` allowing a local test environment for sending messages from an mqtt client app to the `mqtt_server` which will push it to the `influxdb2` instance running in docker locally


- NOTE: only a single message format is supported right now, with ~~no~~ little error handling:
  ```
  topic/f/measurement.field_key {"value": <f64>}
  ```
  where `topic` and `/f/` are ignored, `measurement` can be any string, `field_key` can be any string, and `<f64>` can be any 64 bit floating point number.

example message taken from the logging output:
```
T = test/f/test.test, P = [b"{\n  \"value\":23\n}"]
```

## Building the package documentation
```
cargo doc --open --no-deps
```

## Build and Run the Server
From repo root
```
cargo run
```

## Build and Run in Docker using docker-compose
Ensure `bootstrap.sh` has been run at least once from `tools` directory.

From the root directory (repo root)
```
docker-compose up -d
```

### Debugging in the container
Run bash terminal in the container that is running, if the container completely fails to run, uncomment the `debug lines` from the `mqtt-to-influxdb/Dockerfile` (be sure to comment out the other `ENTRYPOINT` and `CMD`), this will ensure the container just pings forever so you can get into it and dig around the environment.
```
docker exec -it mqtt_server bash
```

# TODO
- ~~handle garbage topic~~
- ~~handle garbage payload~~
- ~~unit tests~~
- ~~an actual logger~~

