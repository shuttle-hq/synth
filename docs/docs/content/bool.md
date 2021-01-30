## Content Family: Bool

#### Content: Bool::Probabilistic

A probabilistic Boolean type of content which with a single parameter. A floating point number which represents the uniform probability of it being `true`

###### Example

```json
"is_user_happy" : {
    "frequency": 0.2,
    "type": "bool"
}
```

###### Example Output

```json
[
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": true
    }
  ]

```

#### Content: Bool::Constant

A constant boolean type. This will always evaluate to either true or false.

###### Example

```json
"is_user_happy" : {
    "constant": false,
    "type": "bool"
}
```

###### Example Output

```json
[
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    },
    {
      "is_user_happy": false
    }
  ]

```