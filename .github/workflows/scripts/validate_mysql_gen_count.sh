#!/bin/bash

sum_rows_query="SELECT (SELECT count(*) FROM hospitals) +  (SELECT count(*) FROM doctors) + (SELECT count(*) FROM patients)"
sum=`mysql -h $MYSQL_HOST -u root --password=$MYSQL_ROOT_PASSWORD -P $MYSQL_PORT $MYSQL_DATABASE -e "$sum_rows_query" | grep -o '[[:digit:]]*'`
if [ "$sum" -lt "10" ]
then
    exit 1
fi