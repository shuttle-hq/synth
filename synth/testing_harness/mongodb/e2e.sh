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
[ "${CI-false}" == "true" ] || SYNTH="cargo run --bin synth"

COLLECTIONS=(hospitals doctors patients)

ERROR='\033[0;31m'
INFO='\033[0;36m'
DEBUG='\033[0;37m'
NC='\033[0m' # No Color

function help() {
  echo "$1 <command>

commands:
  test-generate|Test generating data to mongodb
  test-import|Test importing from mongodb data
  test-local|Run all test on a local machine using the container from 'up' (no need to call 'up' first)
  up|Starts a local Docker instance for testing
  down|Stops container started with 'up'
  cleanup|Cleanup local files after a test run
" | column -t -L -s"|"
}

function test-generate() {
  echo -e "${INFO}Test generate${NC}"

  for collection in "${COLLECTIONS[@]}"
  do
    docker exec -i $NAME mongo \
        hospital \
        --eval "db.${collection}.drop()" > /dev/null
  done

  $SYNTH generate hospital --to mongodb://localhost:${PORT}/hospital --size 100 || return 1

  for collection in "${COLLECTIONS[@]}"
  do
    docker exec -i $NAME mongoexport \
        --quiet \
        --db hospital \
        --collection "$collection" \
        --forceTableScan \
        --jsonArray \
        | jq 'del(.[]._id)' \
        | diff - "hospital_data/$collection.json" || { echo -e "${ERROR}Generation '$collection' does not match${NC}"; return 1; }
  done
}

function test-import() {
  echo -e "${INFO}Testing import${NC}"

  for collection in "${COLLECTIONS[@]}"
  do
    cat "hospital_data/$collection.json" \
    | docker exec -i $NAME mongoimport \
        --quiet \
        --db hospital \
        --collection "$collection" \
        --jsonArray
  done

  $SYNTH import --from mongodb://localhost:${PORT}/hospital hospital_import || { echo -e "${ERROR}Import failed${NC}"; return 1; }
  diff <(jq --sort-keys . hospital_import/*) <(jq --sort-keys . hospital_master/*) || { echo -e "${ERROR}Import namespaces do not match${NC}"; return 1; }
}

function test-local() {
  up || return 1

  result=0
  test-generate || result=$?
  test-import || result=$?

  down
  cleanup

  echo -e "${DEBUG}Done${NC}"

  return $result
}

function up() {
  echo -e "${DEBUG}Starting container${NC}"
  echo -e "${DEBUG}Running database with container name $NAME on port $PORT${NC}"
  docker run --rm --name $NAME -p $PORT:27017 -d mongo > /dev/null

  wait_count=0
  while [ $(docker logs $NAME 2>&1 | grep -c "Waiting for connections") -lt 1 ]
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
  rm -Rf hospital_import
  rm -Rf .synth
}

case "${1-*}" in
  test-generate)
    test-generate || exit 1
    ;;
  test-import)
    test-import || exit 1
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
