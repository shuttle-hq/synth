drop table if exists types;
drop table if exists unofficial_types;

create table types
(
  id int primary key,
  nully text,
  bool boolean NOT NULL,
  string_char char NOT NULL,
  string_varchar varchar(5) NOT NULL,
  string_text text NOT NULL,
  string_bpchar bpchar(6) NOT NULL,
  string_name name NOT NULL,
  /* string_uuid uuid NOT NULL, */
  /* i64_int2 int2 NOT NULL, */
  i32_int4 int4 NOT NULL,
  i64_int8 int8 NOT NULL,
  f32_float4 float4 NOT NULL,
  f64_float8 float8 NOT NULL,
  /* f64_numeric numeric NOT NULL, */
  date_time_timestamptz timestamptz NOT NULL,
  naive_date_time_timestamp timestamp NOT NULL,
  naive_date_date date NOT NULL
);

create table unofficial_types
(
  bool bool NOT NULL,
  string_character character NOT NULL,
  string_character_varying character varying NOT NULL,
  /* i64_smallint smallint NOT NULL, */
  i32_int int NOT NULL,
  i32_integer integer NOT NULL,
  i64_bigint bigint NOT NULL,
  f32_real real NOT NULL,
  f64_double_precision double precision NOT NULL,
  /* f64_decimal decimal NOT NULL, */
  date_time_timestamp_with_time_zone timestamp with time zone NOT NULL,
  id_serial2 serial2 NOT NULL, -- Does not need a generator
  id_smallserial smallserial NOT NULL, -- Does not need a generator
  id_serial4 serial4 NOT NULL, -- Does not need a generator
  id_serial serial NOT NULL, -- Does not need a generator
  id_serial8 serial8 NOT NULL, -- Does not need a generator
  id_bigserial bigserial NOT NULL -- Does not need a generator
);
