Synth's `series` generator creates streams of events based on different 'processes' (a process here can be an auto-correlated process, a poisson process, a cyclical process etc.).

The `series` generators are used in several different contexts:
 - Creating fake events for event-driven systems
 - Modelling time-independent events like 'orders' on a website or 'requests' made to a web server
 - Modelling seasonal behaviour, like an increase in flight frequency for a given airline over the summer

#### Date Time 

All `series` are modelled on so called 'Naive Date Times' - that is 'Date Times' that do not have a timezone. This can be interpreted as Timestamps in UTC. There is future work to improve functionality to add other chrono types.

The format of a series can be set by using the optional `format` field; if `format` is omitted, the default format is `%Y-%m-%d %H:%M:%S`.

#### Duration

The `series` generators will often make use of durations as generation parameters. A duration as a quantity like '1 hour' or '5.7 milliseconds'.

The `series` generators use [`humantime`](https://docs.rs/humantime/2.1.0/humantime/fn.parse_duration.html) to make it easy to specify human readable quantities like `3hr 5m 2s`.

## incrementing

The `incrementing` series simply increments at a fixed duration. This could be for example a stock ticker.

The `incrementing` series has 2 parameters:
- `start`: The time at which the first event occurs
- `increment`: The increment between two consecutive events

#### Example
Below is an example stock ticker for AAPL sampled at regular intervals every minute. 
```json synth
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 10
    },
    "content": {
        "type": "object",
        "ticker": {
            "type": "string",
            "pattern": "AAPL"
        },
        "timestamp": {
            "type": "series",
			"format" : "%Y-%m-%d %H:%M:%S",
            "incrementing": {
				"start" : "2021-02-01 09:00:00",
				"increment" : "1m"
			}
        },
        "price": {
            "type": "number",
            "subtype" : "f64",
            "range" : {
                "high": 105, 
                "low": 100,
                "step": 0.01
            }
        }
    }
}
```

## poisson

The `poisson` series models independent events which occur at random, but which tend to occur at an average rate when viewed as a group.

One example of a poisson process could be earthquakes occurring during the course of a year, or customers arriving at a store, or cars crossing a bridge etc.

The `poisson` series has 2 parameters:
 - `start`: The time at which the first event occurs
 - `rate`: The average duration between two consecutive events

#### Example
The below is an example HTTP server, which was brought up on a given date and has an average of 1 request every 1 minute.
```json synth
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 10
    },
    "content": {
        "type": "object",
        "ip": {
            "type": "string",
            "faker": {
                "generator": "ipv4"
            }
        },
        "timestamp": {
            "type": "series",
            "format": "%d/%b/%Y:%H:%M:%S",
            "poisson": {
                "start": "10/Oct/2000:13:55:36",
                "rate": "1m"
            }
        },
        "request": {
            "type": "string",
            "categorical": {
                "GET /index.html HTTP/1.0": 10,
                "GET /home.html HTTP/1.0": 5,
                "GET /login.html HTTP/1.0": 3
            }
        },
        "response_code": {
            "type": "number",
            "subtype": "u64",
            "categorical": {
                "200": 95,
                "500": 5
            }
        },
        "response_size": {
            "type": "number",
            "range": {
                "low": 500,
                "high": 3000,
                "step": 1
            }
        }
    }
}
```

## cyclical

The `cyclical` series models events which have a 'cyclical' or 'periodic' frequency. 

For example, the frequency of orders placed in an online store peaks during the day and is at it's lowest during the night.

The `cyclical` series has 4 parameters:
- `start`: The time at which the first event occurs
- `max_rate`: The maximum average duration between two events.
- `min_rate`: The minimum average duration between two events
- `period`: The period of the cyclical series.

#### Example
The below is a minimal example of orders being placed in an online store.

```json synth
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 10
    },
    "content": {
        "type": "object",
        "order_id": {
            "type": "number",
            "id": {}
        },
        "item": {
            "type": "string",
            "categorical": {
                "t-shirt": 4,
                "jeans": 1,
                "jacket": 1,
                "belt": 2
            }
        },
        "timestamp": {
            "type": "series",
            "cyclical": {
                "start": "2021-02-01 00:00:00",
                "period": "1d",
                "min_rate": "10m",
                "max_rate": "30s"
            }
        }
    }
}
```

## zip

The `zip` series combines 2 or more series together by `zipping` the output together. That is, the two series are super imposed.

The `zip` series has 1 parameter:
- `series`: The child series to be zipped together

```json synth
{
    "type": "array",
    "length": {
        "type": "number",
        "constant": 10
    },
    "content": {
        "type": "object",
        "order_id": {
            "type": "number",
            "id": {}
        },
        "item": {
            "type": "string",
            "categorical": {
                "t-shirt": 4,
                "jeans": 1,
                "jacket": 1,
                "belt": 2
            }
        },
        "timestamp": {
            "type": "series",
            "zip": {
                "series": [
                    {
                        "cyclical": {
                            "start": "2021-02-01 00:00:00",
                            "period": "1w",
                            "min_rate": "1m",
                            "max_rate": "1s"
                        }
                    },
                    {
                        "cyclical": {
                            "start": "2021-02-01 00:00:00",
                            "period": "1d",
                            "min_rate": "10m",
                            "max_rate": "30s"
                        }
                    }
                ]
            }
        }
    }
}
```
