---
title: PostgreSQL
---

:::note

Synth PostgreSQL is currently **in beta**.

:::

## Usage

`synth` can use PostgreSQL as a data source or sink. Connecting `synth` to a
PostgreSQL is as simple as specifying a URI during the `import` or `generate`
phase.

### URI format

```bash
postgres://<username>:<password>@<host>:<port>/<schema>
```

## Import

`synth` can import directly from a PostgreSQL database and create a data model
from the database schema.

During import, a new namespace will be created from your database schema, and 
a collection is created for each table in a separate JSON file. 

`synth` will automatically detect things like Primary / Foreign Key 
constraints at import time and update the namespace and collection to 
reflect them.

Other constraints such as nullability or maximum size for `VARCHAR` fields are 
also detected automatically.

Finally `synth` will sample data randomly from every table in order to 
create a more realistic data model - trying to infer bounds  

### Example Import Command

```bash
synth import --from postgres://user:pass@localhost:5432/postgres my_namespace 
```

## Generate

### Example Generation Command

```bash
synth generate --to postgres://user:pass@localhost:5432/ my_namespace
```