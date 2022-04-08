#!/bin/bash
set -e

# container specs
CONTAINER="influxdb2"
USERNAME="tester"
PASSWORD="Yegiazaryan"
ORG="test"
BUCKET="tests"
NETWORK="backend-net"

# default env
ADAFRUITE_FORMAT=true
DB_SERVER_HOST=http://127.0.0.1:8086
DB_ORG=${ORG}
DB_BUCKET=${BUCKET}

# create a network for inter-container coms
docker network disconnect ${NETWORK} ${CONTAINER} || true
echo "disconnecting any existing containers from ${NETWORK}"
docker network remove ${NETWORK} || true
echo "cleaning up any existing network"
docker network create -d bridge backend-net
echo "created network"

# pull down the db if exists
docker-compose down || true
echo "Took down existing db"

# pull up the local db
docker-compose up -d
echo "Brought up new db instance, letting instance warm up..."
IS_UP=1
TEST_STR="Error: failed to determine if"
until [[ "${IS_UP}" == 0 ]]; do
    # user/pass/org/bucket/retention setup
    RET=$(docker exec "${CONTAINER}" influx setup --username "${USERNAME}" --password "${PASSWORD}" --org "${ORG}" --bucket ${BUCKET} --retention 0 -f || true)    
    if [[ "${RET}" != *"${TEST_STR}"* ]]; then
        echo
        echo "Initalized database"
        echo "${RET}"
        IS_UP=0
    else
        echo -n "."
        sleep 5
    fi
done

# get token
TOKEN=$(docker exec ${CONTAINER} influx auth list --user "${USERNAME}" --hide-headers | cut -f 3)
echo "Created token for user"

CONFIG_DIR=../config
ENV="DB_SERVER_TOKEN=${TOKEN}\nADAFRUITE_FORMAT=${ADAFRUITE_FORMAT}\nDB_SERVER_HOST=${DB_SERVER_HOST}\nDB_ORG=${DB_ORG}\nDB_BUCKET=${DB_BUCKET}"
echo "Generated default local testing configuration"
mkdir -p ${CONFIG_DIR}
echo "Created config directory to run server locally"
echo -e ${ENV} > ${CONFIG_DIR}/server.env
echo "Wrote default configuration to file"

# how to explor the DB using influxdb's built in dashboard
echo "################################################################################################"
echo
echo "Navigate to localhost:8086 in your browser and log in with the following credentials: "
echo "username: ${USERNAME}"
echo "password: ${PASSWORD}"
echo
echo "################################################################################################"

