Synth's `object` generator type mirrors JSON's objects. They generate key/value pairs whose keys are strings and values
are any sampled from any other generator type. With an `object`, you can compose simpler generators into compound
generators that reflect complex data structures.

The keys of the JSON object to generate are inlined in the `object` keys (e.g. `"identifier"` and `"name"` below).

#### Example

```json synth
{
  "type": "object",
  "identifier": {
    "type": "number",
    "subtype": "u64",
    "range": {
      "low": 0,
      "high": 100,
      "step": 1
    }
  },
  "name": {
    "type": "string",
    "faker": {
      "generator": "name"
    }
  }
}
```

Values of objects can be any of Synth's generator type (including an other object). In the example above, `"identifier"`
has value a [`number`](/synth/content/number) type and `"name"` has value a [`string`](/synth/content/string) type.

Values of objects can be made *optional* by specifying the `"optional": true` attribute.

#### Example
```json synth
{
  "type": "object",
  "email": {
    "optional": true,
    "type": "string",
    "faker": {
      "generator": "ascii_email"
    }
  }
}
```