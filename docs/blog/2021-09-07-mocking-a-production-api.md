# So you want to mock a production API

API mocking refers to the process of simulating the behaviour of a real API
using a fake replacement.

There are many great use-cases for API mocking. You may for exaple want to mock
and API to eliminate development dependencies between engineering teams. If a
service which is a dependecy to your front-end for example isn't ready - you may
want a placeholder which will unblock your dev team.

API mocking is also really powerful tool when doing integrationt tests against
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
  the API provider's testmode can be a limiting contraint.

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
- let's see what the internet has to say.

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
5 day POC. Interestingly we got a bunch of feedback that mocking OAuth flows or
similar would be really useful outside of any specific API. We may come back to
this at a future date.

## Day 3: Evaluating Implementation Alternatives

And then there was Day 3. We'd done our due diligence to pick a popular yet
underserved API and we'd drilled down on how deep we could go in trying to
faithfully represent the implementation.

## Day 4: Implementing the core functionality

## Day 5: Packaging 