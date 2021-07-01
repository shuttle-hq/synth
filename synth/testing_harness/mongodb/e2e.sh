#!/bin/bash

# ------ Begin Setup ------

# database harness information
PORT=27017
NAME=mongo-synth-harness

# shellcheck disable=SC2064
trap " { docker kill '$NAME'; docker rm '$NAME'; } " EXIT

# go to script directory
cd "$( dirname "${BASH_SOURCE[0]}" )/synth" || exit 1
# delete leftover config
rm -f .synth/config.toml

synth init || exit 1

docker run -p $PORT:27017 --name $NAME -d mongo:latest || exit 1

echo "Running database with container name $NAME on port $PORT"

sleep 1

# ------ End Setup ------

# ------ Start Test Generate > Mongo ------
echo "Running generation tests..."

# Generate data into Mongo
synth generate hospital --size 100 --to mongodb://localhost:27017/hospital || exit 1

cd ..

COLLECTIONS=(hospitals doctors patients)

# Export collection and compare against golden master
# We use jq for redacting the MongoDB OID.
for collection in "${COLLECTIONS[@]}"
do
  docker exec -i "$NAME" mongoexport \
      --db hospital \
      --collection "$collection" \
      --forceTableScan \
      --jsonArray \
      | jq 'del(.[]._id)' \
      | diff - "hospital_master_data/$collection.json" || exit 1

done

# ------ End Test Generate > Mongo ------

# ------ Start Test Import < Mongo ------
echo "Running import tests..."

cd synth || exit 1

synth import --from mongodb://localhost:27017/hospital hospital_temp

diff hospital_temp hospital_master

rm -r hospital_temp || exit 1
# ------ End Test Import < Mongo ------

echo "Done..."