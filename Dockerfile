FROM rust:latest
WORKDIR /usr/src/backend

COPY . .

# debug lines
# RUN apt update -y
# RUN apt install iputils-ping -y

RUN cargo build

ENV DB_SERVER_HOST=http://influxdb2:8086
ENTRYPOINT [ "cargo" ]
CMD ["run", "mqtt_server"]

# debug line
# CMD ["ping", "localhost"]

