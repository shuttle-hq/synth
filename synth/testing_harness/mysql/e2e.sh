#!/bin/bash

DB_HOST=127.0.0.1
DB_PORT=3306
DB_USER=root
DB_PASSWORD="mysecretpassword"
DB_NAME=test_db
DB_SCHEME="${MARIA_DB_SCHEME:=mysql}"
CONTAINER_NAME=mysql-synth-harness

######### Initialization #########

# Install dependencies
#apt-get install -y mysql-client

cd "$( dirname "${BASH_SOURCE[0]}" )"

# delete leftover config
rm -f .synth/config.toml

# 0. init workspace
synth init || exit 1

# 1. Init DB service
container_id=$(docker run --name $CONTAINER_NAME -e MYSQL_ROOT_PASSWORD=$DB_PASSWORD -e MYSQL_DATABASE=$DB_NAME -p $DB_PORT:3306 -d $DB_SCHEME:latest)

# Waits til DB is ready
while ! mysql -h $DB_HOST -u root --password=$DB_PASSWORD -P $DB_PORT $DB_NAME -e "SELECT 1" > /dev/null 2>&1; do
    sleep 1
done

######### Export Test #########

# 2. Populate DB schema
mysql -h $DB_HOST -u root --password=$DB_PASSWORD -P $DB_PORT $DB_NAME < 0_hospital_schema.sql || exit 1

result=0

# 3. Verify gen to DB crates min. expected rows
synth generate hospital_master --to $DB_SCHEME://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME --size 1 || result=1
sum_rows_query="SELECT (SELECT count(*) FROM hospitals) +  (SELECT count(*) FROM doctors) + (SELECT count(*) FROM patients)"
sum=`mysql -h $DB_HOST -u root --password=$DB_PASSWORD -P $DB_PORT $DB_NAME -e "$sum_rows_query" | grep -o '[[:digit:]]*'`
[ "$sum" -gt "30" ] || result=1

######### Import Test #########

# 4. Clear out export data and insert known golden master data
mysql -h $DB_HOST -u root --password=$DB_PASSWORD -P $DB_PORT $DB_NAME < 0_hospital_schema.sql || exit 1
mysql -h $DB_HOST -u root --password=$DB_PASSWORD -P $DB_PORT $DB_NAME < 1_hospital_data.sql || exit 1

# 5. Import with synth and diff
synth import --from $DB_SCHEME://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME hospital_import || result=1
diff <(jq --sort-keys . hospital_import/*) <(jq --sort-keys . hospital_master/*) || result=1

######### Cleanup #########

docker rm -f "${container_id}"
rm -rf hospital_import
rm -rf  .synth

# fail if any of the commands failed
if [ $result -ne 0 ]
then
    exit 1
fi
