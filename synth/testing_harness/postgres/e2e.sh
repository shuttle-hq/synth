#!/bin/bash
# Run tests

# database harness information
PORT=5433
PASSWORD="mysecretpassword"
NAME=postgres-synth-harness

# go to script directory
cd "$( dirname "${BASH_SOURCE[0]}" )"
# delete leftover config
rm -f .synth/config.toml

# 0. init workspace
synth init || exit 1

# 1. generate test
echo "Running generate test"
synth generate --size 10 hospital_master > hospital_data_generated.json || exit 1

# check by diff against golden master
diff hospital_data_generated.json hospital_data_generated_master.json || exit 1
rm hospital_data_generated.json

# 2. database import test
docker build -t $NAME . || exit 1
echo "Running database with container name $NAME on port $PORT with password $PASSWORD"
CONTAINER=$(docker run -p $PORT:5432 -e POSTGRES_PASSWORD=$PASSWORD -d $NAME)

sleep 3

RESULT=0

echo "Importing hospital_import namespace from hospital"
synth import --from postgres://postgres:${PASSWORD}@127.0.0.1:$PORT/postgres hospital_import || RESULT=1

echo "stopping container"
docker stop "${CONTAINER}" > /dev/null

# check by diff against golden master
diff <(jq --sort-keys . hospital_import/*) <(jq --sort-keys . hospital_master/*) || RESULT=1

# removing generated namespace files
rm -f hospital_import/doctors.json hospital_import/hospitals.json hospital_import/patients.json || RESULT=1
rmdir hospital_import

rm -f .synth/config.toml
rmdir .synth

# fail if any of the commands failed
if [ $RESULT -ne 0 ]
then
    exit 1
fi
