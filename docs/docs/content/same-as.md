---
title: same_as
---

Synth's `same_as` generator type establishes a relation between two generators. It lets you re-use a value generated at
a different level in say, an [`object`](object), at a different level in the same object. It is often
used to specify foreign key relationships in complex datasets.

A full path is needed at all times. So to refer to the `name` in a file named `example.json`, we need to use
`example.content.name`.

#### Example

```json synth[example.json]
{
  "type": "object",
  "name": {
    "type": "string",
    "faker": {
      "generator": "first_name"
    }
  },
  "same_name": {
    "type": "same_as",
    "ref": "example.content.name"
  }
}
```

The `"ref"` field must point to another existing field. Complex objects can be traversed by concatenating levels
with a period `.` as seen in the following `home.json` file.

#### Example

```json synth[home.json]
{
  "type": "object",
  "address": {
    "type": "object",
    "street_name": {
      "type": "string",
      "faker": {
        "generator": "street_name"
      }
    },
    "zip_code": {
      "type": "string",
      "faker": {
        "generator": "zip_code"
      }
    }
  },
  "same_zip_code": {
    "type": "same_as",
    "ref": "home.content.address.zip_code"
  }
}
```

The `same_as` generator can also be simply declared by the value of the `"ref"` field prefixed with `@`:

```json synth[home2.json]
{
  "type": "object",
  "address": {
    "type": "object",
    "street_name": {
      "type": "string",
      "faker": {
        "generator": "street_name"
      }
    },
    "zip_code": {
      "type": "string",
      "faker": {
        "generator": "zip_code"
      }
    }
  },
  "same_zip_code": "@home2.content.address.zip_code"
}
```
