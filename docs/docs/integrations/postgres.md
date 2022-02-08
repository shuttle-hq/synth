---
title: PostgreSQL
---

:::note

The Synth PostgreSQL integration is currently **in beta**.

:::

## Usage

`synth` can use [PostgreSQL](https://www.postgresql.org/) as a data source or
sink. Connecting `synth` to a PostgreSQL is as simple as specifying a URI
and schema during the `import` or `generate`
phase.

### URI format

```bash
postgres://<username>:<password>@<host>:<port>/<catalog>
```

## Import

`synth` can import directly from a [PostgreSQL](https://www.postgresql.org/)
database and create a data model from the database schema. During import, a
new [namespace](../getting_started/core-concepts#namespaces)
will be created from your database schema, and
a [collection](../getting_started/core-concepts#collections) is created for each
table in a separate JSON file. `synth` will map database columns to fields in
the collections it creates. It then provides default generators for every
collection. Synth will default to the `public` schema but this can be
overriden with the `--schema` flag.

`synth` will automatically detect primary key and foreign key constraints at
import time and update the namespace and collection to reflect them. **Primary
keys** get mapped to `synth`'s [id](../content/number#id)
generator, and **foreign keys** get mapped to the [same_as](../content/same-as.md)
generator.

Finally `synth` will sample data randomly from every table in order to create a
more realistic data model by automatically inferring bounds on types.

`synth` has its own internal data model, and so does Postgres, therefore a
conversion occurs between `synth` types and Postgres types. The inferred type
can be seen below. The synth types link to default generator *variant*
generated during the `import` process for that PostgreSQL type.

Note, not all PostgreSQL types have been covered yet. If there is a type you
need, [open an issue](https://github.com/getsynth/synth/issues/new?assignees=&labels=New+feature&template=feature_request.md&title=)
on GitHub.

<!---
table formatter: https://codebeautify.org/markdown-formatter
-->

| PostgreSQL Type | Synth Type                                              |
| --------------- | ------------------------------------------------------- |
| Null \| T       | [one_of](../content/one-of)<[null](../content/null), T> |
| boolean         | [bool](../content/bool#frequency)                       |
| char            | [string](../content/string#pattern)                     |
| varchar(x)      | [string](../content/string#pattern)                     |
| text            | [string](../content/string#pattern)                     |
| bpchar(x)       | [string](../content/string#pattern)                     |
| name            | [string](../content/string#pattern)                     |
| int2            | [i64](../content/number#range)                          |
| int4            | [i32](../content/number#range)                          |
| int8            | [i64](../content/number#range)                          |
| float4          | [f32](../content/number#range)                          |
| float8          | [f64](../content/number#range)                          |
| numeric         | [f64](../content/number#range)                          |
| timestamptz     | [date_time](../content/date-time)                |
| timestamp       | [naive_date_time](../content/date-time)          |
| date            | [naive_date](../content/date-time)               |
| uuid            | [string](../content/string#uuid)                        |

### Example Import

Below is an example import for a single table.

Postgres table definition:
```sql
create table doctors
(
    id          int primary key,
    hospital_id int not null,
    name        varchar(255) not null,
    date_joined date,
    constraint hospital_fk
    	foreign key(hospital_id)
    		references hospitals(id)
);
```

And the corresponding `synth` collection:
```json synth[expect = "unknown field: hospitals"]
{
  "type": "array",
  "length": {
    "type": "number",
    "range": {
      "low": 0,
      "high": 2,
      "step": 1
    },
    "subtype": "u64"
  },
  "content": {
    "type": "object",
    "date_joined": {
      "type": "one_of",
      "variants": [
        {
          "weight": 1.0,
          "type": "date_time",
          "format": "%Y-%m-%d",
          "subtype": "naive_date",
          "begin": null,
          "end": null
        },
        {
          "weight": 1.0,
          "type": "null"
        }
      ]
    },
    "hospital_id": {
      "type": "same_as",
      "ref": "hospitals.content.id"
    },
    "id": {
      "type": "number",
      "id": {},
      "subtype": "u64"
    },
    "name": {
      "type": "string",
      "pattern": "[a-zA-Z0-9]{0, 255}"
    }
  }
}
```
### Example Import Command

```bash
synth import --from postgres://user:pass@localhost:5432/postgres --schema
main my_namespace
```

### Example

## Generate

`synth` can generate data directly into your PostgreSQL database. First `synth`
will generate as much data as required, then open a connection to your database,
and then perform batch insert to quickly insert as much data as you need.

`synth` will also respect primary key and foreign key constraints, by performing
a [topological sort](https://en.wikipedia.org/wiki/Topological_sorting) on the
data and inserting it in the right order such that no constraints are violated.

### Example Generation Command

```bash
synth generate --to postgres://user:pass@localhost:5432/ --schema
main my_namespace
```
