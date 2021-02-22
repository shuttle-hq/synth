---
title: bank_db
---

# Financial Services Example

In this scenario, we want to create a realistic replica of our production database called `bank_db` for the purpose of
application testing.

## Example Data

`bank_db` has two tables:

1. The `users` table which has all the information pertaining to our customers.

```json
{
  "id": 1,
  "created_at_date": "2009-03-19",
  "created_at_time": "01:00:02",
  "credit_card": "346215176014733",
  "currency": "GIP",
  "email": "tracy31@gmail.com",
  "is_active": false,
  "last_login_at": "2020-06-22T05:46:41+0000",
  "num_logins": 19,
  "password_hash": "eebed079e19dcf5b936e8ca5a648bee38e30bec0212...",
  "username": "tracy31@gmail.com"
}
```

2. The `transactions` table which has transactions referring to the customers in the `users` table:

```json
{
  "id": 1,
  "amount": 5001.70,
  "currency": "GIP",
  "timestamp": "2020-05-13T20:48:01+0000",
  "user_id": 1
}
```

## Feeding Data into Synth

We first create a new workspace to import our dataset into Synth:

```bash
$ mkdir synth_workspace && cd synth_workspace && synth init
```

Synth supports importing from JSON files - to create a namespace we point Synth to our JSON file and run the `import`
command.

```bash
$ synth import bank_db/ --from /path/to/the/file.json
```

```json
{
  "users": [
    {
      "id": 1,
      "created_at_date": "2009-03-19",
      "created_at_time": "01:00:02",
      "credit_card": "346215176014733",
      "currency": "GIP",
      "email": "tracy31@gmail.com",
      "is_active": false,
      "last_login_at": "2020-06-22T05:46:41+0000",
      "num_logins": 19,
      "password_hash": "eebed079e19dcf5b936e8ca5a648bee38e30bec02129790eabdd7084919d7972",
      "username": "tracy31@gmail.com"
    },
    {
      "id": 2,
      "created_at_date": "1989-08-02",
      "created_at_time": "14:01:03",
      "credit_card": "347805284578857",
      "currency": "RON",
      "email": "lunamark@green.com",
      "is_active": false,
      "last_login_at": "2020-06-09T12:25:34+0000",
      "num_logins": 96,
      "password_hash": "5fd241333135a2852c124a28401460d6c7bbd1851a256becddd95f9141e8a74b",
      "username": "lunamark@green.com"
    },
    {
      "id": 3,
      "created_at_date": "1993-04-12",
      "created_at_time": "18:28:28",
      "credit_card": "347652531949402",
      "currency": "EUR",
      "email": "nicholsonjoseph@whitehead.org",
      "is_active": false,
      "last_login_at": "2020-07-24T18:30:45+0000",
      "num_logins": 56,
      "password_hash": "2a3e0851d34b71088fb77c20b840689169b89acef2e74ae687fc1b99387e100b",
      "username": "nick_j"
    },
    {
      "id": 4,
      "created_at_date": "2001-09-11",
      "created_at_time": "06:25:41",
      "credit_card": "341034072996090",
      "currency": "KHR",
      "email": "fullerangela@williams.biz",
      "is_active": false,
      "last_login_at": "2020-03-02T08:31:01+0000",
      "num_logins": 39,
      "password_hash": "10ab7dcd6b7ac468bd442febf904f4025f38017f847774f91ca2bdbdfe029ce1",
      "username": "fullerangela"
    }
  ],
  "transactions": [
    {
      "id": 1,
      "amount": 5001.7,
      "currency": "GIP",
      "timestamp": "2020-05-13T20:48:01+0000",
      "user_id": 1
    },
    {
      "id": 2,
      "amount": 274.4,
      "currency": "GIP",
      "timestamp": "2020-04-07T20:19:23+0000",
      "user_id": 1
    },
    {
      "id": 3,
      "amount": 6199.9,
      "currency": "KHR",
      "timestamp": "2020-02-03T11:24:36+0000",
      "user_id": 2
    },
    {
      "id": 4,
      "amount": 3747.6,
      "currency": "KHR",
      "timestamp": "2020-04-02T02:37:22+0000",
      "user_id": 2
    },
    {
      "id": 5,
      "amount": 4358.4,
      "currency": "KHR",
      "timestamp": "2020-03-20T04:12:11+0000",
      "user_id": 2
    },
    {
      "id": 6,
      "amount": 6597.5,
      "currency": "EUR",
      "timestamp": "2020-09-16T07:26:02+0000",
      "user_id": 3
    }
  ]
}
```

At this stage, we can run the `tree` command to see how the `synth import` sub-command updated our workspace.

```bash
$ tree -a
.
├── bank_db
│   ├── transactions.json
│   └── users.json
└── .synth
    └── config.toml
```

The directory `bank_db` (remember from [Core Concepts](/getting_started/core-concepts) a subdirectory in a workspace represents a
namespace) was created automatically as well as the two collections - `transactions` and `users`.

We can now generate data from our namespace using the `synth generate` sub-command. (We are piping this
into [`jq`](https://stedolan.github.io/jq/download/) for the auto-formatting but this is optional.)

```bash
$ synth generate bank_db/ | jq
{
  "transactions": [
    {
      "amount": 5336.4,
      "currency": "k1BFV",
      "id": 1,
      "timestamp": "kfAuUrNEb8dgGT5",
      "user_id": 2
    }
  ],
  "users": [
    {
      "created_at_date": "Mg",
      "created_at_time": "Me2kBYEDb",
      "credit_card": "LnafugyfWMLf8Gns",
      "currency": "8e9u8h5KYg",
      "email": "yLXGebLNS5ZmZWifCqv20",
      "id": 3,
      "is_active": false,
      "last_login_at": "gH7zB0nkU0ScpmOhWr3vm",
      "num_logins": 78,
      "password_hash": "OJUsm0b4d",
      "username": "3ouuDKRsR7a"
    },
    ...
  ]
}
```

Notice, that the data generated has the right schema, but looks kind of useless. For example the `timestamp` field is
not even a timestamp, it's just a random string.

The semantic meaning of the data has not been perfectly captured by the Synth inference engine.
As `synth` evolves, inference will get better - but for now, we need to tweak the schema.

## Tweaking the Schema

To modify the schema, open the workspace in your favourite editor. Let's take a look at `bank_db/transactions.json`
first.

```json synth
{
  "type": "array",
  "length": {
    "type": "number",
    "subtype": "u64",
    "range": {
      "low": 1,
      "high": 6,
      "step": 1
    }
  },
  "content": {
    "type": "object",
    "amount": {
      "optional": false,
      "type": "number",
      "subtype": "f64",
      "range": {
        "low": 274.4,
        "high": 6597.5,
        "step": 1.0
      }
    },
    "id": {
      "optional": false,
      "type": "number",
      "subtype": "u64",
      "range": {
        "low": 1,
        "high": 6,
        "step": 1
      }
    },
    "timestamp": {
      "optional": false,
      "type": "string",
      "pattern": "[a-zA-Z0-9]*"
    },
    "user_id": {
      "optional": false,
      "type": "number",
      "subtype": "u64",
      "range": {
        "low": 1,
        "high": 3,
        "step": 1
      }
    },
    "currency": {
      "optional": false,
      "type": "string",
      "pattern": "[a-zA-Z0-9]*"
    }
  }
}
``` 

There is quite a bit going on here, so let's break it down. This file represents a schema for
a [`collection`](/getting_started/core-concepts). Collections are [Array::Array](/content/array.md)s under the hood and so they
have 2 fields.

1) The `content` of an Array. This can be any valid JSON, but since `bank_db` originates from a SQL database with column
   names and so on, it is a JSON object.

2) The `length` of an Array. The length of an Array is actually also a Content node. This gives you flexibility - for
   example you can make the length of an array be a `Number::Range`

For more information on how to compose schemas, see the [Schema](/getting_started/schema.md) page.

### Tweaking Individual Fields

Reading through the schema, we can see that Synth inferred `id` as being a `Number::Range`.

What we actually need, is for `id` to be a monotonically increasing [`Number::Id`](/content/number.md) type starting
at `0`.

```json synth
{
  "type": "number",
  "subtype": "u64", 
  "id": {
    "start_at": 0
  }
}
```

The `amount` field is almost right. Synth inferred the right `low` and `high` bounds, but, the step should be `0.01` as
we are dealing with currencies. So let's replace the `amount` field:

```json synth
{
  "type": "number",
  "subtype": "f64",
  "range": {
    "low": 274.4,
    "high": 6597.5,
    "step": 0.01
  }
}
```

Next, we see Synth detected the `timestamp` field as a string following a random pattern. Consulting the documentation
it should be a [String::DateTime](/content/string).

```json synth
{
  "type": "string",
  "date_time": {
    "format": "%Y-%m-%dT%H:%M:%S%z",
    "begin": "2000-01-01T00:00:00+0000",
    "end": "2020-01-01T00:00:00+0000"
  }
}
```

The `user_id` field should point to a valid entry in the `users` collection, so let's use
the [SameAs::SameAs](/content/same-as) content type to express this foreign key relationship.

```json
{
  "type": "same_as",
  "ref": "users.content.id"
}
```

Finally, the `currency` field should reflect the real currencies that the bank supports. We could use
the [String::Faker](/content/string) support `currency_code` generator to do this, but the bank only supports `USD`
, `GBP` and `EUR`. So she uses a [String::Categorical](/content/string)  instead. Roughly 80% of transactions are
in `USD` so let's assign a higher probability to that variant.

```json synth
{
  "type": "string",
  "categorical": {
    "USD": 8,
    "GBP": 1,
    "EUR": 1
  }
}
```

Now let's generate data from the `transactions` collection again:

```bash
$ synth generate bank_db --collection transactions --size 10 | jq
[
  {
    "amount": 1458.2,
    "currency": "GBP",
    "id": 0,
    "timestamp": "2014-12-15T22:49:23+0000",
    "user_id": 3
  },
  {
    "amount": 6043.2,
    "currency": "USD",
    "id": 1,
    "timestamp": "2002-10-10T23:41:32+0000",
    "user_id": 1
  },
  {
    "amount": 2515.7000000000003,
    "currency": "GBP",
    "id": 2,
    "timestamp": "2000-07-17T05:50:27+0000",
    "user_id": 3
  },
...
]
```

Ah, much better.

As an exercise for the reader, try to do the same with the collection `users.json`.
