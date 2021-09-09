---
title: PostgreSQL
---

:::note

The Synth PostgreSQL integration is currently **in beta**.

:::

## Usage

`synth` can use [PostgreSQL](TODO) as a data source or sink. Connecting `synth`
to a PostgreSQL is as simple as specifying a URI during the `import`
or `generate`
phase.

### URI format

```bash
postgres://<username>:<password>@<host>:<port>/<schema>
```

## Import

`synth` can import directly from a [PostgreSQL](TODO) database and create a data
model from the database schema. During import, a new [namespace](TODO) will be
created from your database schema, and a [collection](TODO) is created for each
table in a separate JSON file. `synth` will map database columns to fields 
in the collections it creates. It then provides default generators for every 
collection (see [Type Mapping](TODO) below).

`synth` will automatically detect things like
[primary key](TODO) and [foreign key](TODO) constraints at import time and 
update the namespace and
collection to reflect them. Other constraints such as field nullability or
maximum size for [VARCHAR](TODO) fields are also detected automatically.

Finally `synth` will sample data randomly from every table in order to create a
more realistic data model by doing things like trying to infer bounds on number
ranges.

### Example Import Command

```bash
synth import --from postgres://user:pass@localhost:5432/postgres my_namespace 
```

## Generate

`synth` can generate data directly into your PostgreSQL database. First `synth`
will generate as much data as require, then open a connection to your database,
and then perform batch insert to quickly insert as much data as you need.

`synth` will also respect primary key and foreign key constraints, by performing
a [topologoical sort](TODO_wikipedia) on the data and inserting it in the right
order such that no constraints are violated.

### Example Generation Command

```bash
synth generate --to postgres://user:pass@localhost:5432/ my_namespace
```

## Type Mapping

`synth` has its own internal data model, and so does Postgres, therefore a
conversion occurs between `synth` types and Postgres types. Not we haven't
exhaustively covered all Postgres Types yet.

| PostgreSQL Type | Synth Type      | Default Generator |
|-----------------|-----------------|-------------------|
| Null            | Null            | NullContent        | // Can you even have a Null Column type?
| boolean         | Bool            | BoolContent        |
| char            | String          | StringContent::Pattern([a-zA-Z0-9]{0, 1})|
| varchar(x)      | String          | StringContent::Pattern([a-zA-Z0-9]{0, x})|
| text            | String          | StringContent::Pattern([a-zA-Z0-9]{0, 1})|
| bpchar(x)       | String          | StringContent::Pattern([a-zA-Z0-9]{0, x})|
| name            | String          | StringContent::Pattern([a-zA-Z0-9]{0, 1})|
| int2             | Number::I64     | NumberContent::I64::Range| //todo this is wrong in the implementation
| int4             | Number::I32     | NumberContent::I32::Range|
| int8             | Number::I64     | NumberContent::I64::Range|
| float4             | Number::F32     | NumberContent::F32::Range|
| float8             | Number::F64     | NumberContent::F64::Range|
| numeric             | Number::F64     | NumberContent::F64::Range|
| timestamptz             | ChronoValue::DateTime     | StringContent::DateTime::DateTime|
| timestamp             | ChronoValue::NaiveDateTime     | StringContent::DateTime::NaiveDateTime|
| date             | ChronoValue::NaiveDate     | StringContent::DateTime::NaiveDate|
| uuid             | String     | StringContent::Uuid|