DROP TABLE IF EXISTS arrays;
DROP TABLE IF EXISTS unofficial_arrays;

CREATE TABLE arrays
(
    boolean_array boolean[] NOT NULL,
    char_array char[] NOT NULL,
    varchar_array varchar[] NOT NULL,
    text_array text[] NOT NULL,
    bpchar_array text[] NOT NULL,
    name_array name[] NOT NULL,
    /* uuid_array uuid[] NOT NULL, */
    int2_array int2[] NOT NULL,
    int4_array int4[] NOT NULL,
    int8_array int8[] NOT NULL,
    numeric_array numeric[] NOT NULL,
    uint2_array int2[] NOT NULL,
    uint4_array int4[] NOT NULL,
    uint8_array int8[] NOT NULL,
    unumeric_array numeric[] NOT NULL,
    float4_array float4[] NOT NULL,
    float8_array float8[] NOT NULL,
    int_array_2d int[][] NOT NULL,
    timestamp_array timestamp[] NOT NULL,
    timestamptz_array timestamptz[] NOT NULL,
    date_array date[] NOT NULL,
    time_array time[] NOT NULL,
    json_array json,
    jsonb_array jsonb
);

CREATE TABLE unofficial_arrays
(
    bool_array bool[] NOT NULL,
    character_array character[] NOT NULL,
    character_varying_array character varying[] NOT NULL,
    smallint_array smallint[] NOT NULL,
    int_array int[] NOT NULL,
    integer_array integer[] NOT NULL,
    bigint_array bigint[] NOT NULL,
    usmallint_array smallint[] NOT NULL,
    uint_array int[] NOT NULL,
    uinteger_array integer[] NOT NULL,
    ubigint_array bigint[] NOT NULL,
    real_array real[] NOT NULL,
    double_precision_array double precision[] NOT NULL,
    decimal_array decimal[] NOT NULL,
    timestamp_with_time_zone_array timestamp with time zone[] NOT NULL
);
