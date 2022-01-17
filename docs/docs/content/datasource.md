---
title: datasource
---
Synth's `datasource` generator is used to pull data from an external source. The data can be simple values like a string,
number, or booleans as well as complex values like an array or object.

The `path` option is a URI like the `--from` option for the [import command](/docs/getting_started/command-line#command-import).
Currently only JSON files and Postgres are supported, and hence only the `json:` and `postgres:` schemes are valid. For
any file, the path is relative to where `synth generate` is run from.

The `cycle` is optional and defaults to `false`. It allows you to read a datasource from the beginning once it has been
exhausted when set to `true`.

The `query` option is required when pulling data from any database source. The option defines the query used for fetching
each item from the database.

The `schema` option only works for a Postgres `path` to optionally change the schema the `query` will be run against.

### JSON
When pulling from a JSON file, the JSON is expected to be an array with every item being the value for a single Synth
generator. The following is a valid JSON datasource:

```json[addresses.json]
[
  "21 Mary Street",
  "5 Diascia Avenue",
  "1062 Hill Crescent"
]
```

When generating more than 3 items `cycle` will need to be `true` for this datasource.

#### Example

```json synth
{
  "type": "datasource",
  "path": "json:addresses.json",
  "cycle": true
}
```

### Postgres
Pulling from Postgres will need a query of the data to pull. Unlike a JSON source, the output will always be an object
where the column names will be keys.

#### Example

```json synth
{
  "type": "datasource",
  "path": "postgres://user:password@db-address:5432",
  "query": "SELECT name FROM doctors",
  "schema": "anonymized"
}
```
