---
title: unique
---

Synth's `unique` generator type generates values which are guaranteed to be unique from its child generator.

#### Example

```json synth
{
  "type": "array",
  "length": {
    "type": "number",
    "constant": 10
  },
  "content": {
    "type": "object",
    "ticker": {
      "type": "unique",
      "content": {
        "type": "number",
        "range": {
          "low": 0,
          "high": 10,
          "step": 1
        }
      }
    }
  }
}
```

The unique generator works by trying the inner generator repeatedly until it receives a value which it hasn't seen yet.

By default, the unique generator will give up if it sees the same value more than 64 times but this value can be specified using the `retries` property.

Below is an example of a generator which will fail since the inner generator cannot generate 20 distinct values.

```json synth
{
  "type": "array",
  "length": {
    "type": "number",
    "constant": 20
  },
  "content": {
    "type": "object",
    "ticker": {
      "type": "unique",
      "content": {
        "type": "number",
        "range": {
          "low": 0,
          "high": 10,
          "step": 1
        }
      }
    }
  }
}
```
