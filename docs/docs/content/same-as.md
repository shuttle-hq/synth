---
id: same-as
title: same_as
---

## Content Family: SameAs
#### Content: SameAs

`SameAs` content let's the value of one field follow the value of another field in the Schema.

It can be used to specify datasets with foreign keys. It can also be used (in rarer cases) to create data which is semantically duplicated like in the example below.


###### Example
```json
"field_a": {
    "range": {
        "high": 10,
        "low": 0,
        "step": 1
    },
    "subtype": "u64",
    "type": "number"
},
"follow_a": {
    "type": "same_as",
    "ref": "transactions.content.field_a"
}
```

###### Example Output
```json
[
    {
      "field_a": 5,
      "follow_a": 5
    },
    {
      "field_a": 8,
      "follow_a": 8
    },
    {
      "field_a": 4,
      "follow_a": 4
    },
    {
      "field_a": 4,
      "follow_a": 4
    },
    {
      "field_a": 3,
      "follow_a": 3
    }
  ]

```
