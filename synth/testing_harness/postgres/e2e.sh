#!/usr/bin/env bash

set -uo pipefail

# load env vars
if [ -f .env ]
then
  set -a
  . .env
  set +a
fi

SYNTH="synth"
[ "${CI-false}" == "true" ] || SYNTH="cargo run --quiet --bin synth"

ERROR='\033[0;31m'
INFO='\033[0;36m'
DEBUG='\033[0;37m'
NC='\033[0m' # No Color

function help() {
  echo "$1 <command>

commands:
  load-schema [--no-data]|Fills DB with test schema - defaults to loading data too
  test-generate|Test generating data to postgres
  test-import|Test importing from postgres data
  test-complete|Test generating and importing all types to/from postgres
  test-warning|Test integer warnings
  test-arrays|Test encoding array values
  test-local|Run all test on a local machine using the container from 'up' (no need to call 'up' first)
  up|Starts a local Docker instance for testing
  down|Stops container started with 'up'
  cleanup|Cleanup local files after a test run
" | column -t -L -s"|"
}

function load-schema() {
  docker exec -i $NAME psql -q postgres://postgres:$PASSWORD@localhost:5432/postgres < 0_hospital_schema.sql || return 1
  [ ${1} == "--no-data" ] || docker exec -i $NAME psql -q postgres://postgres:$PASSWORD@localhost:5432/postgres < 1_hospital_data.sql || return 1
}

function test-generate() {
  echo -e "${INFO}Test generate${NC}"
  load-schema --no-data || { echo -e "${ERROR}Failed to load schema${NC}"; return 1; }
  $SYNTH generate hospital_master --to postgres://postgres:$PASSWORD@localhost:$PORT/postgres --size 30 || return 1

  sum_rows_query="SELECT (SELECT count(*) FROM hospitals) +  (SELECT count(*) FROM doctors) + (SELECT count(*) FROM patients)"
  sum=`docker exec -i $NAME psql -tA postgres://postgres:$PASSWORD@localhost:5432/postgres -c "$sum_rows_query"`
  [ "$sum" -gt "30" ] || { echo -e "${ERROR}Generation did not create more than 30 records${NC}"; return 1; }
}

function test-import() {
  echo -e "${INFO}Testing import${NC}"
  load-schema --all || { echo -e "${ERROR}Failed to load schema${NC}"; return 1; }
  $SYNTH import --from postgres://postgres:${PASSWORD}@localhost:${PORT}/postgres hospital_import || { echo -e "${ERROR}Import failed${NC}"; return 1; }
  diff <(jq --sort-keys . hospital_import/*) <(jq --sort-keys . hospital_master/*) || { echo -e "${ERROR}Import namespaces do not match${NC}"; return 1; }
}

function test-complete() {
  rm -Rf complete_import

  echo -e "${INFO}Test complete${NC}"
  docker exec -i $NAME psql -q postgres://postgres:$PASSWORD@localhost:5432/postgres -c "DROP DATABASE IF EXISTS complete" || { echo -e "${ERROR}Failed to drop complete database${NC}"; return 1; }
  docker exec -i $NAME psql -q postgres://postgres:$PASSWORD@localhost:5432/postgres -c "CREATE DATABASE complete" || { echo -e "${ERROR}Failed to create complete database${NC}"; return 1; }
  docker exec -i $NAME psql -q postgres://postgres:$PASSWORD@localhost:5432/complete < 0_complete_schema.sql || { echo -e "${ERROR}Failed to load complete schema${NC}"; return 1; }
  $SYNTH generate complete_master --to postgres://postgres:$PASSWORD@localhost:$PORT/complete || return 1

  sum_rows_query="SELECT (SELECT count(*) FROM types)"
  sum=`docker exec -i $NAME psql -tA postgres://postgres:$PASSWORD@localhost:5432/complete -c "$sum_rows_query"`
  [ "$sum" -eq "10" ] || { echo -e "${ERROR}Generation did not create 10 records${NC}"; return 1; }

  $SYNTH import --from postgres://postgres:${PASSWORD}@localhost:${PORT}/complete complete_import || { echo -e "${ERROR}Importing complete failed${NC}"; return 1; }
  diff <(jq --sort-keys . complete_import/*) <(jq --sort-keys . complete_import_master/*) || { echo -e "${ERROR}Import complete namespaces do not match${NC}"; return 1; }
}

function test-warning() {
  echo -e "${INFO}Testing warnings${NC}"
  docker exec -i $NAME psql postgres://postgres:$PASSWORD@localhost:5432/postgres < warnings/0_warnings.sql
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

function test-arrays() {
  echo -e "${INFO}Testing arrays to postgres${NC}"
  psql -c "CREATE DATABASE arrays;" postgres://postgres:$PASSWORD@localhost:$PORT/postgres
  psql -f arrays/0_arrays.sql postgres://postgres:$PASSWORD@localhost:$PORT/arrays
  ERRORS=$($SYNTH generate --to postgres://postgres:$PASSWORD@localhost:$PORT/arrays arrays 2>&1)
  if [ ! -z "$ERRORS" ]
  then
    echo -e "${ERROR}Did not expect errors:${NC}"
    echo -e $ERRORS
    return 1
  fi

  echo -e "${INFO}Testing importing postgres arrays${NC}"
  $SYNTH import --from postgres://postgres:${PASSWORD}@localhost:${PORT}/arrays arrays_import || { echo -e "${ERROR}Array import failed${NC}"; return 1; }
  diff <(jq --sort-keys . arrays_import/*) <(jq --sort-keys . arrays_master/*) || { echo -e "${ERROR}Import arrays do not match${NC}"; return 1; }
}

function test-local() {
  up || return 1

  result=0
  test-generate || result=$?
  test-import || result=$?
  test-complete || result=$?
  test-warning || result=$?
  test-arrays || result=$?

  down
  cleanup

  echo -e "${DEBUG}Done${NC}"

  return $result
}

function up() {
  echo -e "${DEBUG}Starting container${NC}"
  echo -e "${DEBUG}Running database with container name $NAME on port $PORT with password $PASSWORD${NC}"
  docker run --rm --name $NAME -p $PORT:5432 -e POSTGRES_PASSWORD=$PASSWORD -d postgres > /dev/null

  wait_count=0
  while ! docker exec -i $NAME psql postgres://postgres:$PASSWORD@localhost:5432/postgres -c "SELECT 1" > /dev/null 2>&1
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
  docker rm $NAME > /dev/null
}

function cleanup() {
  echo -e "${DEBUG}Cleaning up local files${NC}"
  rm -Rf hospital_import
  rm -Rf complete_import
  rm -Rf arrays_import
  rm -Rf .synth
}

case "${1-*}" in
  load-schema)
    load-schema ${2---all} || exit 1
    ;;
  test-generate)
    test-generate || exit 1
    ;;
  test-import)
    test-import || exit 1
    ;;
  test-complete)
    test-complete || exit 1
    ;;
  test-warning)
    test-warning || exit 1
    ;;
  test-arrays)
    test-arrays || exit 1
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
