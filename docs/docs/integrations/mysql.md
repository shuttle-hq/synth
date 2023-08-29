---
title: MySQL
---

:::note

The Synth MySQL integration is currently **in beta**.

:::

## Usage

`synth` can use [MySQL](https://www.mysql.org/) as a data source or
sink. Connecting `synth` to a MySQL is as simple as specifying a URI
and schema during the `import` or `generate`
phase.

### URI format

```bash
mysql://<username>:<password>@<host>:<port>/<catalog>
```
One [quirk](https://github.com/launchbadge/sqlx/issues/846) with the 
current MySQL integration is that the host can't be represented in IPv4,
so DNS or IPv6 representations should be used instead. 
If IPv4 compatibility is wanted, use [IPv4-mapped IPv6](https://serverfault.com/a/1102261).

## Import

`synth` can import directly from a [MySQL](https://www.mysql.org/)
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

`synth` has its own internal data model, and so does MySQL, therefore a
conversion occurs between `synth` types and MySQL types. The inferred type
can be seen below. The synth types link to default generator *variant*
generated during the `import` process for that PostgreSQL type.

Note, not all MySQL types have been covered yet. If there is a type you
need, [open an issue](https://github.com/getsynth/synth/issues/new?assignees=&labels=New+feature&template=feature_request.md&title=)
on GitHub.

<!---
table formatter: https://codebeautify.org/markdown-formatter
-->

| MySQL Type | Synth Type                                                  |
| --------------- | ------------------------------------------------------- |
| char            | [string](../content/string#pattern)                     |
| varchar(x)      | [string](../content/string#pattern)                     |
| text            | [string](../content/string#pattern)                     |
| enum            | [string](../content/string#pattern)                     |
| int             | [i32](../content/number#range)                          |
| integer         | [i32](../content/number#range)                          |
| tinyint         | [i8](../content/number#range)                           |
| bigint          | [i64](../content/number#range)                          |
| serial          | [u64](../content/number#range)                          |
| float           | [f32](../content/number#range)                          |
| double          | [f64](../content/number#range)                          |
| numeric         | [f64](../content/number#range)                          |
| decimal         | [f64](../content/number#range)                          |
| timestamp       | [date_time](../content/date-time)                |
| datetime        | [naive_date_time](../content/date-time)          |
| date            | [naive_date](../content/date-time)               |
| time            | [naive_time](../content/date-time)                          |

```
### Example Import Command

```bash
synth import --from mysql://user:pass@localhost:3306/mysql --schema
main my_namespace
```

### Example

## Generate

`synth` can generate data directly into your MySQL database. First `synth`
will generate as much data as required, then open a connection to your database,
and then perform batch insert to quickly insert as much data as you need.

`synth` will also respect primary key and foreign key constraints, by performing
a [topological sort](https://en.wikipedia.org/wiki/Topological_sorting) on the
data and inserting it in the right order such that no constraints are violated.

### Example Generation Command

```bash
synth generate --to mysql://user:pass@localhost:3306/ --schema
main my_namespace
```
