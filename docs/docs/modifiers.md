---
slug: /modifiers
title: Modifiers
---

Modifiers are attributes that can be added to any [generator](content/null) to modify their behavior.

## `optional`

The `optional` modifier makes a generator nullable. It accepts a single boolean value (true or false). 

```json synth
{
  "type": "string",
  "pattern": "hello|goodbye",
  "optional": true
}
```

## `unique`

The `unique` modifiers ensures a generator only outputs non-repeating values. It accepts a single boolean value (true or false). 

```json synth
{
  "type": "array",
  "length": 10,
  "content": {
    "type": "number",
    "subtype": "u64",
    "unique": true,
    "range": {
      "low": 0,
      "high": 20,
      "step": 1
    }
  }
}
```