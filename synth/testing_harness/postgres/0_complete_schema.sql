drop table if exists types;

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
)
