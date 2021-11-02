---
title: Hands-On Tutorial - Creating fake user data in GA to train your number-generating skills 
image: https://storage.googleapis.com/getsynth-public/media/users.jpg
description: In this tutorial we're going to create fake GA data with synth!
---

![Logs](img/users.jpg)

In this tutorial, we will show you that even if the target data looks tricky at first glance, with [`synth`](http://getsynth.com), there is always a solution. We will focus on numeric data.

Together, we will create fake Google Analytics data imitating the behavior of individual website users at the session level, also known as [User Explorer](https://support.google.com/analytics/answer/6339208?hl=en#zippy=%2Cin-this-article). 

For those who actually work with GA: this kind of data may help your data science team in testing user profiling algorithms for e-commerce or online campaigning.

## Prerequisite: Install Synth
If you have not yet done so, you need to get `synth` running on your machine. It can be installed with a single line from a CLI tool of your choice as shown [here](https://getsynth.com/download).

## Elaborating a Data Model

In the GA GUI, a target report would like like this:
| User ID      | Sessions | Avg. Session Duration | Bounce rate | Revenue | Transactions | Goal Conversion Rate
| ----------- | ----------- | ----------- | ----------- | ----------- | ----------- | ----------- |
| fcnMA33      | 64       |  00:06:43 | 1.56% | $0.00 | 0 | 96.88% |
| LAhfn66   | 48        | 00:07:59 | 2.08% | $0.00 | 0 | 33.33% |

Before we start writing a code, let's look at the data closely. 

In our case, in the table we see values in different formats. In reality though, we can limit them to just two data types: integers and floating numbers.

Integer columns are:
* Sessions
* Transactions 

Floating type columns are:
* Bounce rate 
* Avg. Session Duration
* Revenue
* Goal Conversion Rate

For all columns, we need numbers that are equal or greater than zero. Besides, for the percentage columns, we need to introduce the upper limit. The duration column will be expressed in minutes since it is a bit more intuitive than hours and seconds. Additionally, we need to specify a reasonable maximum duration as well. 

:::note

`revenue` and `transactions` have a dependency: a revenue can only be above zero if at least one transaction took place. \
You will need to filter your data later and generate additional rows if not enough data has been left.

:::

The same data in JSON format:

```json
{
    "users":[
        {
        "user id": "fcnMA33",
        "sessions": 64,
        "avg session duration": 6.71,
        "bounce rate": 1.56,
        "revenue": 0.00,
        "transactions": 0,
        "goal conversion rate": 96.886
        },
        {
        "user id": "LAhfn66",
        "sessions": 48,
        "avg session duration": 7.99,
        "bounce rate": 2.08,
        "revenue": 0.00,
        "transactions": 0,
        "goal conversion rate": 33.33
        }
    ]
}
```

In the `synth` [number generators reference](/docs/content/number), you will find out that the tool offers one generator for both types of numbers but the output can be customized using different arguments and parameters.


## Creating Namespace and Collection

Similarly to inserting data into a database, we need to create a "bucket" first:
* a [namespace](/docs/getting_started/core-concepts#namespaces) where you will store your data [schemas](/docs/getting_started/schema): a new directory on your storage drive
* a [collection](/docs/getting_started/core-concepts#collections) for each schema: a JSON file

In your CLI tool, execute the following commands:

```commandline
$ mkdir fake_ga
```
```commandline
$ cd fake_ga && touch users.json
```

Now, we can start coding the data model.

## Coding Your Data Model

You need to open the `users.json` file in a code editor and reproduce the data model.

```json synth
{
    "type": "array",
    "length": 2,
    "content": {
        "type": "object",
        "user id": {},
        "sessions": {},
        "avg session duration": {},
        "bounce rate": {},
        "revenue": {},
        "transactions": {},
        "goal conversion rate": {}
    }
}
```
Then, let's fill in the fields with their generators.

### User ID: A String Generator With Pattern

We will use [this generator](/docs/content/string#pattern) to create user IDs. We can make them look GA-authentic by specifying a regex pattern:

```json synth
{
    "type": "string",
    "pattern": "[a-zA-Z]{5}[0-9]{2}"
}
```
### A Number Generator With a Range
This generator is explained [here](/docs/content/number#range). It allows to define the lower and the upper limits as well as the step: a minimum difference between two values. 

The `number` generator also has a `subtype` parameter that allows to tell `synth` directly what kind of numeric data you need. The values for this parameter come from the [Rust programming language](https://doc.rust-lang.org/reference/types/numeric.html). Examples:
* use `i32`, `i64`, or other subtypes to get integers
* use `f32` or `f64` to get floating numbers

Alternatively, simply enter the data in a desired format into the data model, and `synth` will figure out what you want it to generate.

---
**note**
By default, `synth` will exclude the upper limit value from the dataset. To include the maximum value you need, either increase it or use the `include_high` parameter and set it to `true`.

---

### Integer Columns
#### Sessions (The Number of Sessions)
The number of sessions can be between zero and eternity but we will add the upper limit of 1.000.
```json synth
{
  "type": "number",
  "range": {
      "low": 0,
      "high": 1000,
      "step": 1,
      "include_high": true
  }
}
```
With a `subtype` parameter:
```json synth
{
  "type": "number",
  "subtype": "i32",
  "range": {
      "low": 0,
      "high": 1000,
      "step": 1,
      "include_high": true
  }
}
```
#### Transactions
Let's keep it close to reality and set a maximum here, too.
```json synth
{
  "type": "number",
  "range": {
      "low": 0,
      "high": 100,
      "step": 1,
      "include_high": true
  }
}
```
### Floating Number Columns
Floating numbers will be generated as soon as you enter a floating number in the `step` parameter. 
#### Bounce Rate and Goal Conversion Rate
We could have converted percentages to indexes and set both fields to a range between `0` and `1` but we will keep it as appears in the original report. 
```json synth
{
  "type": "number",
  "range": {
      "low": 0,
      "high": 100,
      "step": 0.01,
      "include_high": true
  }
}
```
#### Average Session Duration
The value `0.017` is a result of our decision to use minutes: 1/60 = 0.01(6). 
```json synth
{
  "type": "number",
  "range": {
      "low": 0.017,
      "high": 10,
      "step": 0.017,
      "include_high": true
  }
}
```
#### Revenue
```json synth
{
  "type": "number",
  "range": {
      "low": 0,
      "high": 10.000,
      "step": 10,
      "include_high": true
  }
}
```
### Putting It All Together
Add the generators to `users.json`. 
## Executing Your Data Model
In your CLI tool, run the data model:
```commandline
synth generate users.json fake_ga
```
In our case, `users.json` is optional since we have only one collection. If you have many in the same namespace and want to generate data only using one schema, you can specify a collection as in the code snippet above.

Your data will be printed to the console. Have a look at this:

```json
{
   "users":[
      {
         "avg session duration":0.272,
         "bounce rate":0.26,
         "goal conversion rate":0.21,
         "revenue":0.7493000000000001,
         "sessions":850,
         "transactions":14,
         "user id":"yCWXw25"
      },
      {
         "avg session duration":8.347,
         "bounce rate":0.5,
         "goal conversion rate":0.47000000000000003,
         "revenue":0.8706,
         "sessions":926,
         "transactions":44,
         "user id":"UTqsk63"
      }
   ]
}
```