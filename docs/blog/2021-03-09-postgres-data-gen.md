---
title: How to Create PostgreSQL Test Data
author: Christos Hadjiaslanis
author_title: Founder
author_url: https://github.com/getsynth
author_image_url: https://avatars.githubusercontent.com/u/14791384?s=460&v=4
tags: [postgres, test data, data generation, tutorial, beginners guide]
description: This post covers three different ways to generate test data for your Postgres database
image: https://i.imgur.com/mErPwqL.png
hide_table_of_contents: false
---

## Introduction

Developing high quality software inevitably requires some testing data.

You could be:
- Integration testing your application for correctness and regressions
- Testing the bounds of your application in your QA process
- Testing the performance of queries as the size of your dataset increases

Either way, the software development lifecycle requires testing data as an integral part of developer workflow. In this article, we'll be exploring 3 different methods for generating test data for a Postgres database.


## Setup

In this example we'll be using Docker to host our Postgres database.

To get started you'll need to [install docker](https://docs.docker.com/get-docker/) and start our container running Postgres:

```bash
% docker run -p 5432:5432 -d -e POSTGRES_PASSWORD=1234 -e POSTGRES_USER=postgres -e POSTGRES_DB=dev postgres
```

As you can see, we've set very insecure default credentials. This is *not* meant to be a robust / productionised instance, but it'll do for our testing harness.


## Our Schema

In this example we'll setup a very simple schema. We're creating a basic app where we have a bunch of companies, and those companies have contacts.
 
```sql
CREATE TABLE companies(
   company_id SERIAL PRIMARY KEY,
   company_name VARCHAR(255) NOT NULL
);

CREATE TABLE contacts(
   contact_id SERIAL PRIMARY KEY,
   company_id INT,
   contact_name VARCHAR(255) NOT NULL,
   phone VARCHAR(25),
   email VARCHAR(100),
   CONSTRAINT fk_company
      FOREIGN KEY(company_id) 
	  REFERENCES companies(company_id)
);
``` 

This schema captures some business logic of our app. We have unique primary keys, we have foreign key constraints, and we have some domain-specific data types which have 'semantic meaning'. For example, the random string `_SX Æ A-ii` is not a valid phone number.

Let's get started.

## Manual Insertion

The first thing you can do which works well when you're starting your project is to literally manually insert all the data you need. This involves just manually writing a SQL script with a bunch of `INSERT` statements. The only thing to really think about is the insertion order so that you don't violate foreign key constraints.

```sql
INSERT INTO companies(company_name)
VALUES('BlueBird Inc'),
      ('Dolphin LLC');	   
	   
INSERT INTO contacts(company_id, contact_name, phone, email)
VALUES(1,'John Doe','(408)-111-1234','john.doe@bluebird.dev'),
      (1,'Jane Doe','(408)-111-1235','jane.doe@bluebird.dev'),
      (2,'David Wright','(408)-222-1234','david.wright@dolphin.dev');
```

So here we're inserting directly into our database. This method is straight forward but does not scale when you need more data or the complexity of your schema increases. Also, testing for edge cases requires your hard-coding edge cases in the inserted data - resulting in a linear amount of work for the bugs you want to catch.

|contact_id|company_id|contact_name                    |phone               |email                           |
|----------|----------|--------------------------------|--------------------|--------------------------------|
|1         |1         |John Doe                        |(408)-111-1234      |john.doe@bluebird.dev           |
|2         |1         |Jane Doe                        |(408)-111-1235      |jane.doe@bluebird.dev           |
|3         |2         |David Wright                    |(408)-222-1234      |david.wright@dolphin.dev        |

## Using generate_series to automate the process

Since you're a programmer, you don't like manual work. You like things to be seamless and most importantly automated!

Postgres comes with a handy function called `generate_series` which, ...*drum roll*... generates series! We can use this to generate as much data as we want without writing it by hand.

Let's use `generate_series` to create 100 companies and 100 contacts

```sql
INSERT INTO companies(company_name)
SELECT md5(random()::text)
FROM generate_series(1,100);

INSERT INTO contacts(company_id, contact_name, phone, email)
SELECT id, md5(random()::text), md5(random()::text)::varchar(20), md5(random()::text) 
FROM generate_series(1,100) id;
``` 

|contact_id|company_id|contact_name                    |phone               |email                           |
|----------|----------|--------------------------------|--------------------|--------------------------------|
|1         |1         |81cc02c106b7c30d4e2b032c91cdb75a|d056f1eee1dca55db03c|cd0da2eef81aaa02d6ba15ef4551fb9f|
|2         |2         |d2b0112bc9bbec85c5229a4b4f28a350|07ba86b1dc24cdadfd24|7404f5b502084563f2ac20c29ed0e584|
|3         |3         |64005702ecaff9f489e8074d6a718aae|50db9534b58e0616cd34|3ea36293665aa1ac38e7d6371893046a|
|4         |4         |202e87bc3d0c8c080048b2c0138c709b|65f6ea317bd0f2c950dc|8b8d9b92916f4cf77c38308f6ac4391b|
|5         |5         |8b2fd25d7b95158df5af671cb3255755|3e6ddc67aabe7164ce9a|ed32035400a7500203352f3597d2548f|

We generated 100 companies and contacts here, the types are correct, *but* the output is underwhelming. First of all, every company has exactly 1 contact, and more importantly the actual data looks completely useless. 

If you care about your data being semantically correct (i.e. text in your `phone` column actually being a phone number) we need to get more sophisticated.

We could define functions ourselves to generate names / phone numbers / emails etc, but why re-invent the wheel? 

## Using a data generator like Synth

[Synth](https://github.com/getsynth/synth) is an open-source project designed to solve the problem of creating realistic testing data. It has integration with Postgres, so you won't need to write any SQL.

Synth uses declarative configuration files (just JSON don't worry) to define how data should be generated. To install the `synth` binary refer to the [installation page](/docs/getting_started/installation).

The first step to use Synth is to create a workspace. A workspace is just a directory in your filesystem that tell Synth that this is where you are going to be storing configuration:

```bash
$ mkdir workspace && cd workspace && synth init 
```

Next we want to create a namespace (basically a stand-alone data model) for this schema. We do this by simply creating a subdirectory and Synth will treat it as a separate schema:

```bash
$ mkdir my_app
``` 

Now comes the fun part! Using Synth's configuration language we can specify how our data is generated. Let's start with the smaller table `companies`.

To tell Synth that `companies` is a table (or collection in the Synth lingo) we'll create a new file `app/companies.json`.

```json
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 1
    },
    "content": {
        "type": "object",
        "company_id": {
            "type": "number",
            "id": {}
        },
        "company_name": {
            "type": "string",
            "faker": {
                "generator": "company_name"
            }
        }
    }
}
```

Here we're telling Synth that we have 2 columns, `company_id` and `company_name`. The first is a `number`, the second is a `string` and the contents of the JSON object define the constraints of the data.

If we sample some data using this data model we get the following:

```bash
$ synth generate my_app/ --size 2
{
  "companies": [
    {
      "company_id": 1,
      "company_name": "Campbell Ltd"
    },
    {
      "company_id": 2,
      "company_name": "Smith PLC"
    }
  ]
}
```

Now we can do the same thing for the `contacts` table by create a file `my_app/contacts.json`. Here we have the added complexity of a foreign key constraints to the company table, but we can solve it easily using Synth's [`same_as`](/docs/content/same-as) generator.


```json
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 1
    },
    "content": {
        "type": "object",
        "company_id": {
            "type": "same_as",
            "ref":"companies.content.company_id"
        },
        "contact_name": {
            "type": "string",
            "faker": {
                "generator": "name"
            }
        },
        "phone": {
            "type": "string",
            "faker": {
                "generator": "phone_number",
                "locales": ["en_GB"]
            }
        },
        "email": {
            "type": "string",
            "faker": {
                "generator": "safe_email"
            }
        }
    }
}
``` 

There is quite a bit going on here - to get an in-depth understanding of the synth configuration refer I'd recommend reading the comprehensive docs. There are tons of cool features which this schema can't really explore!

Now we have both our tables data model under Synth, we can generate data into Postgres:

```bash
$ synth generate my_app/ --to postgres://postgres:1234@localhost:5432/dev
```

Taking a look at the company table:

|contact_id|company_id|contact_name                    |phone               |email                           |
|----------|----------|--------------------------------|--------------------|--------------------------------|
|1         |1         |Carrie Walsh                    |+44(0)117 496 0785  |espinozabetty@hotmail.com       |
|2         |2         |Brittany Flores                 |+441632 960 480     |osharp@mcdaniel.com             |
|3         |3         |Tammy Rodriguez                 |01632960737         |brenda82@ward.org               |
|4         |4         |Amanda Marks                    |(0808) 1570096      |hwilcox@gonzalez.com            |
|5         |5         |Kimberly Delacruz MD            |+44(0)114 4960207   |pgarcia@thompson.com            |
|6         |6         |Jordan Williamson               |(0121) 4960483      |jamesmiles@weber.org            |
|7         |7         |Nicholas Williams               |(0131) 496 0974     |fordthomas@gmail.com            |


Much better :)


## Conclusion

We explored 3 different ways to generate data.

- **Manual Insertion**: Is ok to get you started. If your needs are basic it's the path of least effort to creating a working dataset.
- **Postgres generate_series**: This method scales better than manual insertion - but if you care about the contents of your data and have foreign key constraints you'll need to write quite a bit of bespoke SQL by hand.
- [**Synth**](https://github.com/getsynth/synth): Synth has a small learning curve, but to create realistic testing data at scale it reduces most of the manual labour.


In the next post we'll explore how to subset your existing database for testing purposes. And don't worry if you have sensitive / personal data - we'll cover that too. 