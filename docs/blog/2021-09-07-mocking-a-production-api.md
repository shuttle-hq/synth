# So you want to mock a production API

API mocking refers to the process of simulating the behaviour of a real API
using a fake replacement.

There are many great use-cases for API mocking. You may for exaple want to mock
and API to eliminate development dependencies between engineering teams. If a
service which is a dependency to your front-end for example isn't ready - you may
want a placeholder which will unblock your dev team.

API mocking is also really powerful tool when doing integration tests against
3rd party APIs. This can be broken down roughly into functional and non-function
testing:

- Functional Testing: You care about the semantics of the API. It is important
  to be able to make a request and get an appropriate response which takes your
  request into account. Unfortunately API providers often have sub-par testing
  environments which can make this a real pain.
- Non-Functional Testing: You don't really care about the semantics of the API.
  This can be used to speed up integration tests as requests don't need to
  travel to the API providers servers. You may also want to verify your SLAs,
  load test / stress test your systems etc. In this case being rate-limited by
  the API provider's testmode can be a limiting constraint.

At Synth we're building a declarative data generator - we wanted to apply our
data generation engine to mocking a subset of a popular API and see how far we
could get. We wanted to prototype a solution over (roughly) 5 days as a side
project - this blog post is an overview of that journey.

## Day 1: Picking an API

So much to do - so little time. We had decided we wanted to mock a popular API
but didn't know where to start. We knew companies like Stripe have an
excellent [testmode](TODO]) and even an open source http server which you can
use instead.

We decided to ask the internet 'Which API have you been struggling to test
against' on various forums like Reddit, HN and others. Ok so now we wait and see
let's see what the internet has to say.

## Day 2: Choosing Shopify

Surprisingly (or perhaps not) the internet responded. A bunch of people
responded and primarily complained about payment processors (except for Stripe
which was explicitly praised yet again!). A few products and companies came up
repeatedly as being a nightmare to test against. We qualitatively evaluated the
internet's feedback and reviewed documentation from the different APIs mentioned
for ease of implementation (we had 3.5 days left so we couldn't pick anything
too big). In the end we decided to go with the Shopify API!

// TODO write more here about listening to user feedback

Now the Shopify API is pretty big - and we're building a mock server POC from
scratch, so again we chose to narrow down and try to mock a single endpoint
first. We ended up picking the [Events API](TODO) which seemed pretty straight
forward. Looking at the Events API there are three dimensions to consider when
designing our POC solution.

### 1. The Data Model

The Events API returns a JSON payload which is a collection of Events. An
example Event can be seen below:

```json
{
    "arguments": "Ipod Nano - 8GB",
    // Refers to a certain event and its resources.
    "body": null,
    // A text field containing information about the event.,
    "created_at": "2015-04-20T08:33:57-11:00",
    // The date and time (ISO 8601 format) when the event was created.
    "id": 164748010,
    // The ID of the event.
    "desciption": "Received a new order",
    // A human readable description of the event.
    "path": "/admin/orders/406514653/transactions/#1145",
    // A relative URL to the resource the event is for, if applicable.
    "message": "Received a new order",
    // A human readable description of the event. Can contain some HTML formatting.
    "subject_id": 406514653,
    // The ID of the resource that generated the event.
    "subject_type": "Order",
    // The type of the resource that generated the event.
    "verb": "confirmed"
    // The type of event that occurred. 
}
```

Off the bat it's clear that there is some business logic that needs to be
implemented. For example, there is some notion of causality, i.e. an `Order`
cannot be `closed` before its been `placed`. This non-trivial business logic was
good news - it means we can piggy back off some of the complex data generation
logic that's built into Synth.

Since we don't have access to the code that Runs the Shopify API, we have to
simulate the behaviour of the data model. There are varying degrees of depth
into which you can go, and we broke it into 4 levels:

1. Level 1 - **Stub**: Level 1 is just about exposing and endpoint where the
   data on a per element basis 'looks' right. You have the correct types, but
   you don't really care about correctness across elements. For example you care
   that `path` has the correct `subject_id` in the URI, but you don't care that
   a given Order goes from `placed` to `closed` to `re_opened`etc...
2. Level 2 - **Mock**: Level 2 involves maintaining the semantics of the Events
   collection. For example `created_at` should be monotonically increasing as it
   follows `id`. `verb`s should follow proper causality (as per the order
   example above). etc.
3. Level 3 - **Emulate**: Level 3 is about maintaining semantics across
   endpoints. For example creating an order in a different Shopify API endpoint
   should create an `order_placed` event in the Events API.
4. Level 4 - **Simulate**: Here you are basically reverse engineering all the
   business logic of the API. It should be indistinguisable from the real thing.

Really these levels can be seen as increasing in scope as you go simulate
semantics per-element, per-endpoint, cross-endpoint and finally for the entire
API.

### 2. The Endpoint Behaviour

The Events API is not a naive CRUD system. The endpoint exposes various query
parameters (which are basically filters) which alter the response body. Luckly
the Events API has simple behaviour, and as long as their implementation stays
true to the description it should be easy to emulate:

```
limit:          The number of results to show. (default: 50, maximum: 250)
since_id:       Show only results after the specified ID.
created_at_min: Show events created at or after this date and time. (format: 2014-04-25T16:15:47-04:00)
created_at_max: Show events created at or before this date and time. (format: 2014-04-25T16:15:47-04:00)
filter:         Show events specified in this filter.
verb:           Show events of a certain type.
fields:         Show only certain fields, specified by a comma-separated list of field names.
```

### 3. Authentication

We decided not to touch authentication for now as the scope would blow up for a
5-day POC. Interestingly we got a bunch of feedback that mocking OAuth flows or
similar would be really useful outside of any specific API. We may come back to
this at a future date.

## Day 3: Evaluating Implementation Alternatives

And then there was Day 3. We'd done our due diligence to pick a popular yet
underserved API, and we'd drilled down on how deep we could go in trying to
faithfully represent the implementation.

As any self-respecting engineer would do, we decided to scour the internet for
off-the-shelf solutions to automate as much of the grunt work as possible. Some
naive Googling brought up a mock server automation solution
called [JSON server](https://github.com/typicode/json-server) - REST API
automation solution which spins up a rest API for you given a data definition.
Excited by this we quickly wrote up a collection of 2 fake Event API events, and
started JSON server - and it worked!

Well almost; we were initially excited by the fact that it did exactly what is
said on the tin and very well, however it didn't have an easy way to specify the
custom query parameters we needed to faithfully reproduce the API like returning
results before or after a given `created_at` timestamp (feel free to let us know
if we missed something here!).

So we needed something a little more sophisticated. The internet came to the
rescue again with a comprehensive
[list](https://en.wikipedia.org/wiki/Comparison_of_API_simulation_tools) of API
simulation tools. The basic precondition we had was that the API simulator had
to be OSS with a permissive license we could build on. This immediately
disqualified 50% of the available solutions, and we did a divide and conquer
exercise quickly evaluating the rest.

![Api Simulation Tools](media/api-simulation-tools.png)

The remaining tools were either not built for this purpose, or they were
incredibly complex pieces of software that would take a while to get acquainted
with.

In the end we decided to implement the endpoint functionality ourselves - we
figured that a 50 LOC NodeJS server would do a fine job for a POC.

## Day 4: Implementing the core functionality

Day 4 was the most straight forward. Let's get this thing to work!

### 1. The Data Model

We decided to reproduce the API at level 2 since we didn't really have any other
endpoints. We used [synth](https://github.com/getsynth/synth) to quickly whip up
a [data model](TODO_link_to_GH) that generates data that looks like the Shopify
API. I won't go into depth on how this works here as it's been covered
in [other](2021-08-31-seeding-databases-tutorial.md) posts. In about 15 minutes
we had 10 Mb data that looks like this:

```json
...
{
    "arguments": "generate virtual platforms",
    "body": null,
    "created_at": "2019-09-17T14:16:47",
    "description": "Received new order",
    "id": 477306,
    "message": "Received new order",
    "path": "/admin/orders/83672/transactions/#6020",
    "subject_id": 1352997,
    "subject_type": "Order",
    "verb": "closed"
},
{
"arguments": "innovate best-of-breed schemas",
"body": null,
"created_at": "2017-05-20T00:04:41",
"description": "Received new order",
"id": 370051,
"message": "Received new order",
"path": "/admin/orders/82607/transactions/#9154",
"subject_id": 1226112,
"subject_type": "Order",
"verb": "sale_pending"
},
{
"arguments": "incentivize scalable mindshare",
"body": null,
"created_at": "2018-02-21T12:51:36",
"description": "Received new order",
"id": 599084,
"message": "Received new order",
"path": "/admin/orders/01984/transactions/#3595",
"subject_id": 1050540,
"subject_type": "Order",
"verb": "placed"
}
...
```

We then dumped it all in a MongoDB collection with the one-liner:

```bash
$ synth generate shopify --to mongodb://localhost:27017/shopify --size 40000
```

Next step is to re-create the Events API endpoint.

### 2. Creating the API

@brokad

## Day 5: Packaging

// TODO add a picture of Docker here

The data is ready, the API is ready, time to package this thing up and give it
to people to actually use. Let's see if our experiment was a success.

When thinking about distributing our API, we're optimising for two things:

1. Ease of use - how simple it is for someone to download this thing and
   get going
2. Time - we have 2-3 hours to make sure this thing is packaged and ready to go

in that order.

We need to pack the data, database and a NodeJS runtime to actually run the
server. Our initial idea was to use `docker-compose` with 2 services, the
database, web-server and the network plumbing to get it to work.. After
discussing this for a few minutes, we decided that `docker-compose` may be an
off-ramp for some users as they don't have it or are not familiar with how it
works. This went against our tenet which is 'ease of use'.  

So we decided to take the slightly harder and hackier route of packaging the 
whole thing in a single Docker container. It seemed like the best trade-off 
between goals 1 and 2.

There were 6 steps to getting this thing over the line:
1. Start with the MongoDB base image. This gives us a Linux environment and 
   a database.
2. Download and install NodeJS runtime in the container.
3. Download and install Synth in the container.
4. Copy the javacript sources over & the Synth data model
5. Write a small [ENTRYPOINT](TODO) [shell script](TODO_PATH_TO_OUR_SCRIPT) 
   to start the `mongod`, server and generate data into the server
6. Expose port 3000

And we're done! We've ~~hackily~~ happilly packaged our mock API in a 
platform agnostic one liner.

## Was it a success?

An important aspect of this experiment was to see if we could conceive, 
research, design and implement a PoC in a week (on the side, we were working 
on Synth at the same time). And I can safely say this was a success! We got 
it done to spec. An interesting thing to note is that **60%** of the time was 
spent on ideating, researching and planning - and only 40% of the time on the actual 
implementation. However spending all that time before writing code 
definitely saved a bunch of time, and if we didn't the project would have 
overshot or failed.

Now if the PoC itself was a success is a different question. This is where 
you come in. If you're using the Events API, pull the image and play around 
with it - if its helpful to you let us know and we can systematically 
improve and implement the rest of the Shopify API.

Fin.