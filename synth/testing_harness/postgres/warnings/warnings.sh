#!/bin/bash
# Run tests

# database harness information
PORT=5432
PASSWORD="mysecretpassword"
NAME=postgres-synth-harness-warnings

# go to script directory
cd "$( dirname "${BASH_SOURCE[0]}" )"

# 1. set up database for export test
docker build -t $NAME . || exit 1
echo "Running database with container name $NAME on port $PORT with password $PASSWORD"
CONTAINER=$(docker run -p $PORT:5432 -e POSTGRES_PASSWORD=$PASSWORD -d $NAME)

sleep 5

docker ps

# 2. generate test
echo "Running generate test"
synth generate --size 10 . --to postgres://postgres:${PASSWORD}@127.0.0.1:$PORT/postgres

echo "stopping container"
docker stop "${CONTAINER}" > /dev/null
