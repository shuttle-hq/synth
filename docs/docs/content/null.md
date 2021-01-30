## Content Family: Nulls
#### Content: Null

`Null` content is what is says on the tin. Synth will evaluate Null content into a JSON Null primitive.

###### Example

```json
"my_null_field" : {
    "type": "null"
}
```

###### Example Output

```json
[
    {
      "my_null_field": null
    },
    {
      "my_null_field": null
    },
    {
      "my_null_field": null
    },
    {
      "my_null_field": null
    },
    {
      "my_null_field": null
    }
  ]

```