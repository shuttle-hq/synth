drop table if exists f32;

create table f32
(
    float4 float4 NOT NULL,
    float8 float8 NOT NULL,

    -- Unofficial types
    real real NOT NULL,
    double_precision double precision NOT NULL
);
