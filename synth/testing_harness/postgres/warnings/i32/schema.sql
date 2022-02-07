drop table if exists i32;

create table i32
(
    int2 int2 NOT NULL,
    int4 int4 NOT NULL,
    int8 int8 NOT NULL,

    -- Unofficial types
    smallint smallint NOT NULL,
    int int NOT NULL,
    integer integer NOT NULL,
    bigint bigint NOT NULL
);
