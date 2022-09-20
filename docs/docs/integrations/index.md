---
title: Integrations
---

## Integrations

- [MySQL](mysql)
- [PostgreSQL](postgres)
- MongoDB

## Quirks

If a field is named `type`, data generation will fail. This is because it will conflict with the schema definition container object. [The solution](https://github.com/shuttle-hq/synth/issues/1) to this is to use the `type_` key.
