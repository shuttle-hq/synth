## Content Family: Object

#### Content: Object

Objects are basically JSON objects. Object keys have to be strings (they should not contain `.` or whitespace) and values are any `Content`.

###### Example
```json
"user" : {
  "id" : {
    "type": "number",
    "subtype": "u64",
    "id": {
      "start_at" : 0
    }
  },
  "name": {
    "type": "string",
    "faker" : {
        "generator" : "name"
    }
  },
  "type": "object"
}
```

###### Example Output

```json
 [
    {
      "user": {
        "id": 0,
        "name": "Nicole Jones"
      }
    },
    {
      "user": {
        "id": 1,
        "name": "Jason Walker"
      }
    },
    {
      "user": {
        "id": 2,
        "name": "Jonathan Spencer"
      }
    },
    {
      "user": {
        "id": 3,
        "name": "Vanessa Richard"
      }
    },
    {
      "user": {
        "id": 4,
        "name": "David Cohen"
      }
    }
  ]
```