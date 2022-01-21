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
[ "${CI-false}" == "true" ] || { SYNTH="../../../target/debug/synth"; cargo build --bin synth || exit 1; }

ERROR='\033[0;31m'
INFO='\033[0;36m'
DEBUG='\033[0;37m'
NC='\033[0m' # No Color

function help() {
  echo "$1 <command>

commands:
  [test-name]|Optional parameter to run a specific test
  --cleanup|Cleanup local files after a test run
  --help|Shows this help text
" | column -t -L -s"|"
}

function test() {

  result=0
  for ns in generate/*
  do
    test-specific $ns || { result=$?; echo; }
  done

  cleanup

  echo -e "${DEBUG}Done${NC}"

  return $result
}

function test-specific() {
  echo -e "${INFO}Testing $1${NC}"
  MSG=`$SYNTH generate $1 2>&1`
  [ $? == 1 ] || { echo "$MSG"; echo -e "${ERROR}Expected error but got none${NC}"; return 1; }
  diff --ignore-matching-lines="\[.*\]" <(echo "$MSG") "$1/errors.txt" || { echo "$MSG" | grep "\[.*\]"; echo -e "${ERROR}Errors do not match${NC}"; return 1; }
}

function cleanup() {
  echo -e "${DEBUG}Cleaning up local files${NC}"
  rm -Rf hospital_import
  rm -Rf .synth
}

case "${1-all}" in
  all)
    test || exit 1
    ;;
  --cleanup)
    cleanup
    ;;
  --help)
    help $0
    exit 1
    ;;
  *)
    test-specific "generate/$1" || exit 1
esac
