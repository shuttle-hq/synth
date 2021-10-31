#!/bin/bash

set -uo pipefail

# load env vars
if [ -f .env ]
then
  set -a
  . .env
  set +a
fi

SYNTH="synth"

ERROR='\033[0;31m'
INFO='\033[0;36m'
DEBUG='\033[0;37m'
NC='\033[0m' # No Color

function help() {
  echo "$1 <command>

commands:
  load-schema|Fills DB with test schema
  test-generate|Test if 'synth generate' is correct
  test-import|Test importing from postgres data created by 'load-schema'
  test-warning|Test integer warnings
  test-local|Run all test on a local machine using the container from 'up' (no need to call 'up' first)
  up|Starts a local Docker instance for testing
  down|Stops container started with 'up'
  cleanup|Cleanup local files after a test run
" | column -t -L -s"|"
}

function load-schema() {
  psql -f 0_hospital_schema.sql postgres://postgres:$PASSWORD@localhost:$PORT/postgres || return 1
  psql -f 1_hospital_data.sql postgres://postgres:$PASSWORD@localhost:$PORT/postgres || return 1
}

function test-generate() {
  echo -e "${INFO}Test generate${NC}"
  $SYNTH generate hospital_master | jq > hospital_data_generated.json
  diff hospital_data_generated.json hospital_data_generated_master.json || { echo -e "${ERROR}Generated file does not match master${NC}"; return 1; }
}

function test-import() {
  echo -e "${INFO}Testing import${NC}"
  $SYNTH import --from postgres://postgres:${PASSWORD}@localhost:${PORT}/postgres hospital_import || { echo -e "${ERROR}Import failed${NC}"; return 1; }
  diff <(jq --sort-keys . hospital_import/*) <(jq --sort-keys . hospital_master/*) || { echo -e "${ERROR}Import namespaces do not match${NC}"; return 1; }
}

function test-warning() {
  echo -e "${INFO}Testing warnings${NC}"
  psql -f warnings/0_warnings.sql postgres://postgres:$PASSWORD@localhost:$PORT/postgres
  WARNINGS=$($SYNTH generate --size 10 --to postgres://postgres:$PASSWORD@localhost:$PORT/postgres warnings 2>&1)
  if [[ "$WARNINGS" == *"warnings.int32"* && "$WARNINGS" == *"warnings.int64"* ]]
  then
    echo -e "${DEBUG}Expected warnings were emitted${NC}"
  else
    echo -e "${ERROR}Did not get expected warnings:${NC}"
    echo "expected"
    echo "[yyyy-mm-ddThh:MM:ssZ WARN synth::datasource::relational_datasource] Trying to put an unsigned u32 into a int4 typed column warnings.int32"
    echo "[yyyy-mm-ddThh:MM:ssZ WARN synth::datasource::relational_datasource] Trying to put an unsigned u64 into a int8 typed column warnings.int64"
    echo "got"
    echo $WARNINGS
    return 1
  fi
}

function test-local() {
  cargo build --bin synth --release
  SYNTH="../../../target/release/synth"

  up || return 1

  result=0
  test-generate || result=$?
  test-import || result=$?
  test-warning || result=$?

  down
  cleanup

  echo -e "${DEBUG}Done${NC}"

  return $result
}

function up() {
  echo -e "${DEBUG}Starting container${NC}"
  docker build -t $NAME . || return 1
  echo -e "${DEBUG}Running database with container name $NAME on port $PORT with password $PASSWORD${NC}"
  docker run --rm --name $NAME -p $PORT:5432 -e POSTGRES_PASSWORD=$PASSWORD -d $NAME > /dev/null

  wait_count=0
  while [ $(docker logs $NAME 2>&1 | grep -c "ready to accept connections") -lt 2 ]
  do
    range=$(printf "%${wait_count}s")
    echo -en "\\r${DEBUG}Waiting for DB to come up${range// /.}${NC}"
    wait_count=$((wait_count + 1))
    sleep 1
  done
  echo
}

function down() {
  echo -e "${DEBUG}Stopping container${NC}"
  docker stop $NAME > /dev/null
}

function cleanup() {
  echo -e "${DEBUG}Cleaning up local files${NC}"
  rm -f hospital_data_generated.json
  rm -Rf hospital_import
  rm -Rf .synth
}

case "${1-*}" in
  load-schema)
    load-schema || exit 1
    ;;
  test-generate)
    test-generate || exit 1
    ;;
  test-import)
    test-import || exit 1
    ;;
  test-warning)
    test-warning || exit 1
    ;;
  test-local)
    test-local || exit 1
    ;;
  up)
    up
    ;;
  down)
    down
    ;;
  cleanup)
    cleanup
    ;;
  *)
    help $0
    exit 1
esac
