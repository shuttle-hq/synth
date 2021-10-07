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
has value a [`number`](number) type and `"name"` has value a [`string`](string) type.

Values of objects can be made nullable by specifying the `"optional": true` attribute.

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

By default, optional values that are generated as `null` will produce a key-value pair of the form `key: null`. This behavior can be controlled by specifying the `skip_when_null: true` attribute on the object generator.

#### Example
```json synth
{
  "type": "object",
  "skip_when_null": true,
  "email": {
    "optional": true,
    "type": "string",
    "faker": {
      "generator": "ascii_email"
    }
  }
}
```

If a field should have the name `"type"`, this would clash with the predefined object attribute of the same name.
This can be worked around by changing the name to `"type_"`. The additional underscore will be removed in the
generated values.

#### Example

```json synth
{
  "type": "object",
  "type_": {
    "type": "string",
    "categorical": {
      "user": 90,
      "contributor": 8,
      "admin": 2
    }
  }
}
```
