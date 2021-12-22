drop table if exists synth;

create table synth
(
  id int primary key,
  nully text,
  bool boolean NOT NULL,
  string_char char NOT NULL,
  string_varchar varchar(5) NOT NULL,
  string_text text NOT NULL,
  string_bpchar bpchar(6) NOT NULL,
  string_name name NOT NULL,
  string_uuid uuid NOT NULL
)
