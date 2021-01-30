## Content Family: Array

#### Content: Array

Arrays represent, well Arrays. 

They are composed of two parts.

1) The `content` of an Array. Array's in JSON are not bound to being of a specific type, the same is true in the Synth schema.

2) The `length` of an Array. The length of an Array is actually also a Content node. This gives you flexbility - for example you can make the length of an array be a `Number::Range`

The below example will generate an array of between 50 and 100 credit cards.

###### Example
```json
"credit_cards" : {
  "content": {
    "type": "string",
    "faker": {
      "generator": "credit_card_number",
      "card_type": "amex"
    }
  },
  "length": {
      "range": {
          "high": 100,
          "low": 50,
          "step": 1
      },
      "subtype": "u64",
      "type": "number"
  },
  "type": "array"
}
```
