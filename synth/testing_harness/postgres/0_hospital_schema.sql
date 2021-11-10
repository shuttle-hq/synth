drop table if exists patients;
drop table if exists doctors;
drop table if exists hospitals;

create table hospitals
(
    id            int primary key,
    hospital_name varchar(255),
    address       varchar(255)
);

create table doctors
(
    id          int primary key,
    hospital_id int references hospitals (id),
    name        varchar(255),
    date_joined date
);

create table patients
(
    id          int primary key,
    doctor_id   int references doctors (id),
    name        varchar(255),
    date_joined date,
    address     varchar(255),
    phone       varchar(20),
    ssn         varchar(12)
);