---
title: Interactive Tutorial - Creating fake HTTP logs with Synth
---

![Logs](img/logs.jpg)

In this tutorial we're going to be creating fake HTTP logs
with [synth](http://getsynth.com). Whether you're benchmarking some
log-collector or are experimenting with log compression, this tutorial aims to
teach you how to build a customisable data model that will let you generate as
many logs as you need.

## Data Model

We want to generate HTTP logs - but what is an HTTP log actually? Different HTTP
servers will have different logging conventions - but we'll stick to the
[Common Log Format](https://en.wikipedia.org/wiki/Common_Log_Format) (CLF) 
used as a
default by web servers like
the [Apache Web Server](https://en.wikipedia.org/wiki/Apache_HTTP_Server).

CLF has the following syntax:
```
host ident authuser date request status bytes
```
concretely:
```
127.0.0.1 user-identifier frank [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326
```
or as json:
```json
{
    "host":     "127.0.0.1",                    // ip address of the client
    "ident":    "user-identifier",              // RFC 1413 identity of the cient
    "authuser": "frank",                        // userid
    "date":     "10/Oct/2000:13:55:36 -0700",   // %d/%b/%Y:%H:%M:%S %z
    "request":  "GET /apache_pb.gif HTTP/1.0",  // HTTP request from client
    "status":   200,                            // HTTP status code
    "bytes":    2326                            // size of object returned in bytes
}
```

## Installing Synth

To install `synth` head over to the [download page](/download).

## Implementation

So let's get started!

To get started with synth let's create a new [namespace](/docs/getting_started/core-concepts) by simply 
creating a new directory - we'll call it `clf-logs`:
```commandline
$ mkdir clf-logs 
```
Next let's create a [collection](/docs/getting_started/core-concepts#collections) called logs which will define the 
schema of our log data:
```commandline
$ cd clf-logs && touch logs.json
```

### Scaffolding

We're going to base the meat of our CLF log schema on the `date` field. We 
want our logs to look and behave realistically. We can model requests 
arriving at our web server using a [poisson process](https://en.wikipedia.org/wiki/Poisson_point_process) - a model which is often used to model 
independent random events with a mean interval between events (like 
customers arriving a store.) `synth` has a [poisson generator](/docs/content/series#poisson) which we can use to do this.

First let's open up `logs.json` in our favourite IDE and define an array of 
objects which have a field `date`:
```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": 0
    }
}
```

Now lets sanity check our schema by generating some data:
```commandline
$ synth generate . --collection logs | jq
[
  {
    "date": 0
  }
]
```

### Poisson Generator

Cool! We generated a `0`. So far so good. Now let's swap out the `0` for the 
poisson generator in `logs.json`.

```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "10m"
            }
        }
    }
}
```

Our poisson series generator has 2 parameters:
- `start`: the earliest time an event can occur
- `rate`: the average arrival rate for events

### Host

Next let's add the `host` field. Here we can simply use one of our [faker 
generators](/docs/content/string#faker) for generating `ipv4` addresses:
```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "10m"
            }
        },
        "host": {
            "type": "string",
            "faker": {
                "generator": "ipv4"
            }
        }
    }
}
```

Easy!


### Ident

Ident corresponds to the RFC 1413 Identification Protocol. We can leave this 
as `"-"` using the [pattern generator](/docs/content/string#pattern) for now, but you can get 
creative if you need something more elaborate for your use case:


```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "10m"
            }
        },
        "host": {
            "type": "string",
            "faker": {
                "generator": "ipv4"
            }
        },
        "ident": {
            "type": "string",
            "pattern": "-"
        }
    }
}
```

### Authuser

`authuser` is the userid of the person requesting the document. Usually "-" unless .htaccess has requested authentication.

Here we'll just use the `faker` [`first_name`](/docs/content/string#first_name) generator get a bunch of first 
names:

```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "10m"
            }
        },
        "host": {
            "type": "string",
            "faker": {
                "generator": "ipv4"
            }
        },
        "ident": {
            "type": "string",
            "pattern": "-"
        },
        "authuser": {
            "type": "string",
            "faker": {
                "generator": "first_name"
            }
        }
    }
}
```

### Request

Request is a little involved - we're going to need to use the [format](/docs/content/string#format) 
generator to string together multiple generators. `format` takes arbitrarily 
many arguments and a format string. Our format string is of the form:

```
{http_method} {endpoint} HTTP/1.0
```

We can build this compositionally with 2 child generators and a `format` 
generator:

```json synth
{
    "type": "string",
    "format": {
        "format": "{http_method} /{endpoint} HTTP/1.0",
        "arguments": {
            "http_method": {
                "type": "string",
                "categorical": {
                    "GET": 1,
                    "PUT": 1,
                    "POST": 1,
                    "PATCH": 1
                }
            },
            "endpoint": {
                "type": "string",
                "faker": {
                    "generator": "file_name"
                }
            }
        }
    }
}
```
Here `http_method` is a categorical generator, with equal probability to 
yield any of the 4 HTTP methods defined, and `endpoint` is a `faker` 
generator which generates file names.

### Status

For `status` we'll be using the [`categorical`](/docs/content/string#categorical) generator as well - easy:
```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "10m"
            }
        },
        "host": {
            "type": "string",
            "faker": {
                "generator": "ipv4"
            }
        },
        "ident": {
            "type": "string",
            "pattern": "-"
        },
        "authuser": {
            "type": "string",
            "faker": {
                "generator": "first_name"
            }
        },
        "request": {
            "type": "string",
            "format": {
                "format": "{http_method} /{endpoint} HTTP/1.0",
                "arguments": {
                    "http_method": {
                        "type": "string",
                        "categorical": {
                            "GET": 1,
                            "PUT": 1,
                            "POST": 1,
                            "PATCH": 1
                        }
                    },
                    "endpoint": {
                        "type": "string",
                        "faker": {
                            "generator": "file_name"
                        }
                    }
                }
            }
        },
        "status": {
            "type": "number",
            "categorical": {
                "200": 8,
                "404": 1,
                "500": 1
            }
        }
    }
}
```

In this case the we're assigning weights to the variants of the categorical. 
80% of the time we'll be getting `200`, 10% of the time we'll get `404` and 
10% of the time we'll get `500`.

### Bytes

And finally `bytes`. For `bytes` we'll use a [number range](/docs/content/number#range) generator 
from 1 b to 1 MiB:

```json synth
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "object",
        "date": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "10m"
            }
        },
        "host": {
            "type": "string",
            "faker": {
                "generator": "ipv4"
            }
        },
        "ident": {
            "type": "string",
            "pattern": "-"
        },
        "authuser": {
            "type": "string",
            "faker": {
                "generator": "first_name"
            }
        },
        "request": {
            "type": "string",
            "format": {
                "format": "{http_method} /{endpoint} HTTP/1.0",
                "arguments": {
                    "http_method": {
                        "type": "string",
                        "categorical": {
                            "GET": 1,
                            "PUT": 1,
                            "POST": 1,
                            "PATCH": 1
                        }
                    },
                    "endpoint": {
                        "type": "string",
                        "faker": {
                            "generator": "file_name"
                        }
                    }
                }
            }
        },
        "status": {
            "type": "number",
            "categorical": {
                "200": 8,
                "404": 1,
                "500": 1
            }
        },
        "bytes": {
            "type": "number",
            "range": {
                "low": 1,
                "high": 1048576
            }
        }
    }
}
```
And we're done!

## Tying it all together

Now we have our raw data structure - we need to compose it into the proper 
log string. To do this, we'll create a separate collection `formatted` by 
creating another file in our namespace called `formatted.json`. We'll then 
use a combination of the `format` and [`same_as`](/docs/content/same-as) generators to compose 
together fields from our original collection `logs.json`.

```json
{
    "type": "array",
    "length": 1,
    "content": {
        "type": "string",
        "format": {
            "format": "{host} {ident} {authuser} [{date}] {request} {status} {bytes}",
            "arguments": {
                "host": "@logs.content.host",
                "ident": "@logs.content.ident",
                "authuser": "@logs.content.authuser",
                "date": "@logs.content.date",
                "request": "@logs.content.request",
                "status": "@logs.content.status",
                "bytes": "@logs.content.bytes"
            }
        }   
    }
}
```

And we're done!

```console
$ synth generate . --collection formatted --size 5 | jq
[
  "187.123.85.239 - Margarete [10/Oct/2000:13:58:04] GET /woman.7z HTTP/1.0 200 274906",
  "151.58.227.40 - Ewald [10/Oct/2000:14:41:24] POST /oliver.mp3 HTTP/1.0 404 912953",
  "90.255.24.42 - Demarcus [10/Oct/2000:14:51:11] POST /young.doc HTTP/1.0 200 1047925",
  "123.174.213.110 - Carlos [10/Oct/2000:15:07:59] GET /way.rar HTTP/1.0 200 49926",
  "253.115.73.9 - Paolo [10/Oct/2000:15:14:53] PATCH /next.png HTTP/1.0 200 884672"
]
```

