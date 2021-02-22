---
title: same_as
---

Synth's `same_as` generator type establishes a relation between two generators. It lets you re-use a value generated at
a different level in say, an [`object`](/synth/content/object), at a different level in the same object. It is often
used to specify foreign key relationships in complex datasets.

#### Example

```json synth
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
    "ref": "name"
  }
}
```

The `"ref"` field must point to another existing field. Complex objects can be traversed by using concatenating levels
with a period `.`.

#### Example

```json synth
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
        "generator": "zipcode"
      }
    }
  },
  "same_zip_code": {
    "type": "same_as",
    "ref": "address.zip_code"
  }
}
```