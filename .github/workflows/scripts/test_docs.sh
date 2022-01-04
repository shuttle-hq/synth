#!/usr/bin/env bash

###############################################################################
## This script test all 'json synth' code blocks in markdown files.
##
## Each code block can be given a specific filename using 
## 'json synth[filename.json]' which is helpful for 'same_as' examples.
##
## Some code blocks might also be used to show invalid syntax. These can be
## marked with 'json synth[expect = "expected error message"]' to test the
## message does occur.
###############################################################################

ERROR='\033[0;31m'
SUCCESS='\033[0;32m'
DEBUG='\033[0;30m'
INFO='\033[0;37m'
NC='\033[0m' # No Color

function test_file(){
  file=$1

  in_code_block=false
  in_data_file=false
  code_block=""
  file_name=""
  expected_errors=()
  schema_files=0
  line_count=0

  ns=$(basename $file .md)
  mkdir -p "$ns"

  while IFS= read -r line
  do
    line_count=$((line_count+1))

    if [ $(echo "$line" | grep -Pc "^\`\`\`json(\[.*\])?$") -gt 0 ]
    then
      # Copy datasource data files marked with `json[file_name.json]`
      file_name=$(echo "$line" | grep -Po "(?<=\[).*\.json(?=\])")

      # File not marked
      if [ -z "$file_name" ]
      then
        echo -e "${DEBUG}$file:$line_count has a JSON only code block that will be skipped${NC}"
        continue
      fi

      echo -e "${DEBUG}$file:$line_count($file_name) is a JSON data file that will be copied${NC}"
      in_data_file=true
      continue
    fi

    # Find start of code blocks
    if [ "$in_code_block" = false ] && [ "$in_data_file" = false ] && [ $(echo "$line" | grep -Pc "^\`\`\`json synth(\[.*\])?$") -gt 0 ]
    then
      file_name=$(echo "$line" | grep -Po "(?<=\[).*\.json(?=\])")

      if [ -z "$file_name" ]
      then
        file_name="$schema_files.json"
      fi

      expected_error_tmp=$(echo "$line" | grep -Po "(?<=\[expect = \").*(?=\"\])")

      if [ "$expected_error_tmp" != "" ]
      then
        expected_errors+=("$expected_error_tmp")
      fi

      in_code_block=true
      continue
    fi

    if [ "$in_code_block" = false ] && [ "$in_data_file" = false ]
    then
      continue
    fi

    # Find end of active code blocks
    if [ "$line" = "\`\`\`" ]
    then
      if [ "$in_data_file" = false ]
      then
        # Wrap one liners in array
        if [ $(echo -e "$code_block" | wc -l) -lt 3 ]
        then
          code_block=$( echo "{
            \"type\": \"array\",
            \"length\": 1,
            \"content\": {
              \"type\": \"object\",
              $code_block
            }
          }")
        # Wrap not having array type in array
        elif echo -e "$code_block" | sed -n 3p | grep -vq "\"type\": \"array\","
        then
          code_block=$( echo "{
            \"type\": \"array\",
            \"length\": 1,
            \"content\": $code_block
          }")
        fi

        echo -e "$code_block" > "$ns/$file_name"
      else
        echo -e "$code_block" > "$file_name"
      fi

      in_code_block=false
      in_data_file=false
      code_block=""
      file_name=""
      schema_files=$((schema_files+1))
      continue
    fi

    # Strip comments
    # Double up backslashes which `echo -e` will remove when writting to file
    line=$(echo $line | sed "{s#//.*##; s#\\\#\\\\\\\#g}")
    code_block="$code_block\n$line"
  done < $file

  # Only test namespace if it has any files
  if [ $schema_files -gt 0 ]
  then
    output=$(2>&1 1>/dev/null synth generate "$ns")

    if [ "$output" != "" ]
    then
      if [ -z "$expected_errors" ]
      then
        echo -e "${ERROR}$file failed${NC}"
        echo -e "$output"
        return 1
      fi

      expected_errors=$(IFS='|' ; echo "${expected_errors[*]}")

      if [ $(echo "$output" | grep -Pc "($expected_errors)") -lt 1 ]
      then
        echo -e "${ERROR}$file does not have expected errors ($expected_errors)${NC}"
        echo -e "$output"
        return 1
      else
        echo -e "${SUCCESS}$file has expected errors ($expected_errors)${NC}"
      fi
    else
      echo -e "${SUCCESS}$file passed${NC}"
    fi
  else
    echo -e "${INFO}$file has nothing to test${NC}"
  fi
  
  # Cleanup if passed
  rm -r "$ns"
}

mkdir -p tmp
cd tmp

markdown_files=$(find ../ -type d -name node_modules -prune -o -type f -name "*.md")
result=0

for file in $markdown_files
do
  if [ -d $file ]
  then
    continue
  fi

  test_file "$file" || result=$?
done

exit $result
