drop table if exists patients;
drop table if exists doctors;
drop table if exists "Hospitals";

drop type if exists gender;

create table "Hospitals"
(
    id            int primary key,
    hospital_name varchar(255),
    address       varchar(255),
    specialities  varchar(255)[]
);

create table doctors
(
    id          int primary key,
    hospital_id int references "Hospitals" (id),
    name        varchar(255),
    date_joined date
);

create type gender as enum ('male', 'female', 'unspecified');

create table patients
(
    id          int primary key,
    doctor_id   int references doctors (id),
    name        varchar(255),
    gender      gender,
    date_joined date,
    address     varchar(255),
    phone       varchar(20),
    ssn         varchar(12)
);
