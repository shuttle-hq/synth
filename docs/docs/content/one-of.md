---
title: one_of
---

Synth's `one_of` generator is a compound operator, i.e. a way to compose other generator types together. It lets you
define a new generator that samples randomly from a specified list of dependent generators (called _variants_). In that
way, `one_of` is similar to [categorical `string`s](string#categorical). However, the variants of
a `one_of` generator are allowed to be generated from any other Synth generator.

Variants of a `one_of` generator are specified with the `"variants"` field. Allowed value is an array of Synth
generators.

```json synth
{
  "type": "one_of",
  "variants": [
    {
      "weight": 0.5,
      "type": "string",
      "pattern": "M|F"
    },
    {
      "weight": 0.5,
      "type": "null"
    }
  ]
}
```

`one_of` has a concept of a weight for each variant - where weight represents the individual entry's contribution to the
probability distribution. The weight can be specified by adding the `"weight"` field to the corresponding variant's
definition.

#### Example

```json synth
{
  "type": "one_of",
  "variants": [
    {
      "weight": 9.5,
      "type": "string",
      "faker": {
        "generator": "address"
      }
    },
    {
      "weight": 0.5,
      "type": "object",
      "postcode": {
        "type": "string",
        "faker": {
          "generator": "post_code"
        }
      },
      "number": {
        "type": "number",
        "subtype": "u64",
        "range": {
          "low": 1,
          "high": 200,
          "step": 2
        }
      }
    }
  ]
}
```
