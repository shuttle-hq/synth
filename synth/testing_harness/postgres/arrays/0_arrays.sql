DROP TABLE IF EXISTS arrays;

CREATE TABLE arrays
(
    bool_array boolean[],
    char_array char[],
    varchar_array varchar[],
    name_array name[],
    string_array text[],
    smallint_array smallint[],
    int_array int[],
    bigint_array bigint[],
    numeric_array numeric[],
    usmallint_array smallint[],
    uint_array int[],
    ubigint_array bigint[],
    unumeric_array numeric[],
    real_array real[],
    double_precision_array double precision[],
    int_array_2d int[][],
    timestamp_array timestamp[],
    timestamptz_array timestamptz[],
    date_array date[],
    time_array time[],
    json_array json,
    jsonb_array json
);

