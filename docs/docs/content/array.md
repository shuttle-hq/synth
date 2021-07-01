Synth's `array` type mirrors JSON's arrays. An `array` type requires the following fields:

- `"content"`: specifies the elements of the generated array. Allowed value is any of Synth's generator type.
- `"length"`: specifies the length of the generated array. Allowed value is any of
  Synth's [`number`](/synth/content/number) type with `"subtype": "u64"`.

The example below generates arrays of credit card numbers with `3` to `10` elements.

#### Example

```json synth
{
  "type": "array",
  "length": {
    "type": "number",
    "subtype": "u64",
    "range": {
      "high": 10,
      "low": 3,
      "step": 1
    }
  },
  "content": {
    "type": "string",
    "faker": {
        "generator": "credit_card"
    }
  }
}
```