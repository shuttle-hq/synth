## Content Family: Number

Numbers stick to the supported number types in JSON. Synth currently supports `u64`, `i64` and `f64` - and all the `Number::Content` types are generic over these sub-types. The required sub-type can be specified using the `subtype` property. 

#### Content: Number::Range

Defined a range over a semi-open interval `[low, high)` with a `step`.

For example, the range below will generate values in the set: `{-273.15, -273.16, ... 14999999.98, 14999999.99}`

###### Example

```json
"temperature" : {
    "range": {
        "high": 15000000.0, // temperature at sun's core in Celcius
        "low": -273.15,     // 0 Kelvin
        "step": 0.01
    },
    "subtype": "f64",
    "type": "number"
}
```

###### Example Output

```json
[
    {
      "temperature": 13138318.15
    },
    {
      "temperature": 5350910.79
    },
    {
      "temperature": 1581420.62
    },
    {
      "temperature": 6668315.12
    },
    {
      "temperature": 7076333.55
    }
]
```
#### Content: Number::Constant

A constant number type. This will always evaluate to the same number.

###### Example

```json
"pi" : {
    "constant": 3.14159,
    "subtype": "f64",
    "type": "number"
}
```

###### Example Output

```json
[
    {
      "pi": 3.14159
    },
    {
      "pi": 3.14159
    },
    {
      "pi": 3.14159
    },
    {
      "pi": 3.14159
    },
    {
      "pi": 3.14159
    }
]
```
#### Content: Number::Id

A monotonically increasing id. The optional `start` field defaults to 0 if unspecified.

Synth currently supports `u64` ids.

###### Example

```json
"id" : {
    "type": "number",
    "subtype": "u64",
    "id": {
      "start_at" : 10
    } 
}
```

###### Example Output

```json
[
    {
      "id": 10
    },
    {
      "id": 11
    },
    {
      "id": 12
    },
    {
      "id": 13
    },
    {
      "id": 14
    }
]
```