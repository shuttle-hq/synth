Synth's `bool` generator type generates one of two values `true` or `false`.

## constant

A constant `bool` type. This will always evaluate to the specified value.

#### Example

```json synth
{
  "type": "bool",
  "constant": false
}
```

## frequency

A probabilistic `bool` type. The `frequency` parameter (value between `0.` and `1.`) controls the probability of the
generated value being `true`.

#### Example

```json synth
{
  "type": "bool",
  "frequency": 0.5
}
```
