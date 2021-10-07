"use strict";(self.webpackChunksynth_docs=self.webpackChunksynth_docs||[]).push([[1866],{3905:function(e,t,n){n.d(t,{Zo:function(){return p},kt:function(){return u}});var a=n(7294);function i(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function o(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);t&&(a=a.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,a)}return n}function r(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?o(Object(n),!0).forEach((function(t){i(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):o(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function s(e,t){if(null==e)return{};var n,a,i=function(e,t){if(null==e)return{};var n,a,i={},o=Object.keys(e);for(a=0;a<o.length;a++)n=o[a],t.indexOf(n)>=0||(i[n]=e[n]);return i}(e,t);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);for(a=0;a<o.length;a++)n=o[a],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(i[n]=e[n])}return i}var l=a.createContext({}),d=function(e){var t=a.useContext(l),n=t;return e&&(n="function"==typeof e?e(t):r(r({},t),e)),n},p=function(e){var t=d(e.components);return a.createElement(l.Provider,{value:t},e.children)},h={inlineCode:"code",wrapper:function(e){var t=e.children;return a.createElement(a.Fragment,{},t)}},c=a.forwardRef((function(e,t){var n=e.components,i=e.mdxType,o=e.originalType,l=e.parentName,p=s(e,["components","mdxType","originalType","parentName"]),c=d(n),u=i,m=c["".concat(l,".").concat(u)]||c[u]||h[u]||o;return n?a.createElement(m,r(r({ref:t},p),{},{components:n})):a.createElement(m,r({ref:t},p))}));function u(e,t){var n=arguments,i=t&&t.mdxType;if("string"==typeof e||i){var o=n.length,r=new Array(o);r[0]=c;var s={};for(var l in t)hasOwnProperty.call(t,l)&&(s[l]=t[l]);s.originalType=e,s.mdxType="string"==typeof e?e:i,r[1]=s;for(var d=2;d<o;d++)r[d]=n[d];return a.createElement.apply(null,r)}return a.createElement.apply(null,n)}c.displayName="MDXCreateElement"},1760:function(e,t,n){n.r(t),n.d(t,{frontMatter:function(){return s},contentTitle:function(){return l},metadata:function(){return d},assets:function(){return p},toc:function(){return h},default:function(){return u}});var a=n(7462),i=n(3366),o=(n(7294),n(3905)),r=["components"],s={title:"So you want to mock an API",author:"Christos Hadjiaslanis",author_title:"Founder",author_url:"https://github.com/getsynth",author_image_url:"https://avatars.githubusercontent.com/u/14791384?s=460&v=4",tags:["synth","prisma","testing","mocking","story"],description:"This blog post is an overview of a 5 day prototyping journey building a mock API",image:"https://storage.googleapis.com/getsynth-public/media/api.jpg",hide_table_of_contents:!1},l=void 0,d={permalink:"/blog/2021/09/07/mocking-a-production-api",source:"@site/blog/2021-09-07-mocking-a-production-api.md",title:"So you want to mock an API",description:"This blog post is an overview of a 5 day prototyping journey building a mock API",date:"2021-09-07T00:00:00.000Z",formattedDate:"September 7, 2021",tags:[{label:"synth",permalink:"/blog/tags/synth"},{label:"prisma",permalink:"/blog/tags/prisma"},{label:"testing",permalink:"/blog/tags/testing"},{label:"mocking",permalink:"/blog/tags/mocking"},{label:"story",permalink:"/blog/tags/story"}],readingTime:11.68,truncated:!1,authors:[{name:"Christos Hadjiaslanis",title:"Founder",url:"https://github.com/getsynth",imageURL:"https://avatars.githubusercontent.com/u/14791384?s=460&v=4"}],prevItem:{title:"50 ways to crash our product",permalink:"/blog/2021/09/27/crash"},nextItem:{title:"Seeding test databases in 2021 - best practices",permalink:"/blog/2021/08/31/seeding-databases-tutorial"}},p={authorsImageUrls:[void 0]},h=[{value:"Day 1: Picking an API",id:"day-1-picking-an-api",children:[]},{value:"Day 2: Choosing Shopify",id:"day-2-choosing-shopify",children:[{value:"1. The Data Model",id:"1-the-data-model",children:[]},{value:"2. The Endpoint Behaviour",id:"2-the-endpoint-behaviour",children:[]},{value:"3. Authentication",id:"3-authentication",children:[]}]},{value:"Day 3: Evaluating Implementation Alternatives",id:"day-3-evaluating-implementation-alternatives",children:[]},{value:"Day 4: Implementing the core functionality",id:"day-4-implementing-the-core-functionality",children:[{value:"1. The Data Model",id:"1-the-data-model-1",children:[]},{value:"2. Creating the API",id:"2-creating-the-api",children:[]}]},{value:"Day 5: Packaging",id:"day-5-packaging",children:[]},{value:"Was it a success?",id:"was-it-a-success",children:[]}],c={toc:h};function u(e){var t=e.components,s=(0,i.Z)(e,r);return(0,o.kt)("wrapper",(0,a.Z)({},c,s,{components:t,mdxType:"MDXLayout"}),(0,o.kt)("p",null,(0,o.kt)("img",{alt:"So you want to mock an API",src:n(8329).Z})),(0,o.kt)("p",null,"API mocking refers to the process of simulating the behaviour of a real API\nusing a fake replacement."),(0,o.kt)("p",null,"There are many great use-cases for API mocking. You may want to mock an API to\neliminate development dependencies between engineering teams. If for example a\nservice which is a dependency to your front-end isn't ready - you may want a\nplaceholder which will unblock your front-end team."),(0,o.kt)("p",null,"API mocking is also a really powerful tool when doing integration tests against\n3rd party APIs. This can be broken down roughly into functional and non-function\ntesting:"),(0,o.kt)("ul",null,(0,o.kt)("li",{parentName:"ul"},"Functional Testing: You care about the semantics of the API. It is important\nto be able to make a request and get an appropriate response which takes your\nrequest into account. Unfortunately API providers often have sub-par testing\nenvironments which can make this a real pain."),(0,o.kt)("li",{parentName:"ul"},"Non-Functional Testing: You don't really care about the semantics of the API.\nThis can be used to speed up integration tests as requests don't need to\ntravel to the API providers servers. You may also want to verify your SLAs,\nload test / stress test your systems etc. In this case being rate-limited by\nthe API provider's testmode can be a limiting constraint.")),(0,o.kt)("p",null,"At ",(0,o.kt)("a",{parentName:"p",href:"https://getsynth.com"},"Synth"),", we're building a declarative data generator.\nWe wanted to apply our data generation engine to mocking a subset of a popular\nAPI and see how far we could go. We set out to prototype a solution over (roughly)\n5 days as a side project - this blog post is an overview of that journey."),(0,o.kt)("h2",{id:"day-1-picking-an-api"},"Day 1: Picking an API"),(0,o.kt)("p",null,"So much to do, so little time. We decided we wanted to mock a popular API\nbut didn't know where to start. Companies like Stripe have an\nexcellent ",(0,o.kt)("a",{parentName:"p",href:"https://stripe.com/docs/testing"},"testmode")," and even\nan ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/stripe/stripe-mock"},"open source http server")," which you\ncan use instead."),(0,o.kt)("p",null,"We decided to ask the internet 'Which API have you been struggling to test\nagainst' on various forums like Reddit, HN and others. Ok so now we wait and see\nlet's see what the internet has to say."),(0,o.kt)("h2",{id:"day-2-choosing-shopify"},"Day 2: Choosing Shopify"),(0,o.kt)("p",null,"Lo and behold! the internet responded. A bunch of people responded and primarily\ncomplained about payment processors (except for Stripe which was explicitly\npraised yet again!). A few products and companies came up repeatedly as being\ndifficult to test against. We qualitatively evaluated the internet's feedback\nand reviewed documentation from the different APIs mentioned to understand\nthe implementation complexity. After all we had 3.5 days left, so we couldn't\npick anything too complex. In the end we decided to go with the ",(0,o.kt)("a",{parentName:"p",href:"https://shopify.dev/api/"},"Shopify API"),"!"),(0,o.kt)("p",null,"Just as a disclaimer we have absolutely no issues with Shopify, it just so\nhappens that a lot of the feedback we got pointed us that direction."),(0,o.kt)("p",null,"Now the Shopify API is pretty big - and we're building a mock server POC from\nscratch, so we decided to narrow down and try to mock a single endpoint\nfirst. We chose\nthe ",(0,o.kt)("a",{parentName:"p",href:"https://shopify.dev/api/admin/rest/reference/events/event"},"Event API"),"\nwhich seemed pretty straight forward. Looking at the Event API there are three\ndimensions to consider when designing our POC solution."),(0,o.kt)("h3",{id:"1-the-data-model"},"1. The Data Model"),(0,o.kt)("p",null,"The Event API returns a JSON payload which is a collection of Events. An example\nEvent can be seen below:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-json"},'{\n    // Refers to a certain event and its resources.\n    "arguments": "Ipod Nano - 8GB",\n    // A text field containing information about the event.\n    "body": null,\n    // The date and time (ISO 8601 format) when the event was created.\n    "created_at": "2015-04-20T08:33:57-11:00",\n    // The ID of the event.\n    "id": 164748010,\n    // A human readable description of the event.\n    "desciption": "Received a new order",\n    // A relative URL to the resource the event is for, if applicable.\n    "path": "/admin/orders/406514653/transactions/#1145",\n    // A human readable description of the event. Can contain some HTML formatting.\n    "message": "Received a new order",\n    // The ID of the resource that generated the event.\n    "subject_id": 406514653,\n    // The type of the resource that generated the event.\n    "subject_type": "Order",\n    // The type of event that occurred. \n    "verb": "confirmed"\n}\n')),(0,o.kt)("p",null,"Off the bat it's clear that there is some business logic that needs to be\nimplemented. For example, there is some notion of causality, i.e. an ",(0,o.kt)("inlineCode",{parentName:"p"},"Order"),"\ncannot be ",(0,o.kt)("inlineCode",{parentName:"p"},"closed")," before it's been ",(0,o.kt)("inlineCode",{parentName:"p"},"placed"),". This non-trivial business logic was\ngood news - it means we can showcase some complex data generation logic that's\nbuilt into ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/synth"},"synth"),"."),(0,o.kt)("p",null,"Since we don't have access to the code that runs the Shopify API, we have to\nsimulate the behaviour of the Event data model. There are varying degrees of\ndepth into which one can go, and we broke it into 4 levels:"),(0,o.kt)("ol",null,(0,o.kt)("li",{parentName:"ol"},"Level 1 - ",(0,o.kt)("strong",{parentName:"li"},"Stub"),": Level 1 is just about exposing an endpoint where the data\non a ",(0,o.kt)("em",{parentName:"li"},"per element")," basis 'looks' right. You have the correct types, but you\ndon't really care about correctness across elements. For example, you care\nthat ",(0,o.kt)("inlineCode",{parentName:"li"},"path")," has the correct ",(0,o.kt)("inlineCode",{parentName:"li"},"subject_id")," in the URI, but you don't care that\na given Order goes from ",(0,o.kt)("inlineCode",{parentName:"li"},"placed")," to ",(0,o.kt)("inlineCode",{parentName:"li"},"closed")," to ",(0,o.kt)("inlineCode",{parentName:"li"},"re_opened"),"etc..."),(0,o.kt)("li",{parentName:"ol"},"Level 2 - ",(0,o.kt)("strong",{parentName:"li"},"Mock"),": Level 2 involves maintaining the semantics of the Events\n",(0,o.kt)("em",{parentName:"li"},"collection")," as a whole. For example ",(0,o.kt)("inlineCode",{parentName:"li"},"created_at")," should always increase\nas ",(0,o.kt)("inlineCode",{parentName:"li"},"id")," increases\n(a larger ",(0,o.kt)("inlineCode",{parentName:"li"},"id")," means an event was generated at a later date).\n",(0,o.kt)("inlineCode",{parentName:"li"},"verb"),"s should follow proper causality (as per the order example above). etc."),(0,o.kt)("li",{parentName:"ol"},"Level 3 - ",(0,o.kt)("strong",{parentName:"li"},"Emulate"),": Level 3 is about maintaining semantics ",(0,o.kt)("em",{parentName:"li"},"across\nendpoints"),". For example creating an order in a different Shopify API endpoint\nshould create an ",(0,o.kt)("inlineCode",{parentName:"li"},"order_placed")," event in the Event API."),(0,o.kt)("li",{parentName:"ol"},"Level 4 - ",(0,o.kt)("strong",{parentName:"li"},"Simulate"),": Here you are basically reverse engineering all the\nbusiness logic of the API. It should be ",(0,o.kt)("em",{parentName:"li"},"indistinguishable")," from the real\nthing.")),(0,o.kt)("p",null,"Really these levels can be seen as increasing in scope as you simulate\nsemantics per-element, per-endpoint, cross-endpoint and finally for the entire\nAPI."),(0,o.kt)("h3",{id:"2-the-endpoint-behaviour"},"2. The Endpoint Behaviour"),(0,o.kt)("p",null,"The Event API exposes 2 endpoints:"),(0,o.kt)("ul",null,(0,o.kt)("li",{parentName:"ul"},(0,o.kt)("inlineCode",{parentName:"li"},"GET /admin/api/2021-07/events.json")," which retrieves a list of all events"),(0,o.kt)("li",{parentName:"ul"},(0,o.kt)("inlineCode",{parentName:"li"},"GET /admin/api/2021-07/events/{event_id}.json")," which retrieves a single even\nby its ID.")),(0,o.kt)("p",null,"The first endpoint exposes various query parameters (which are basically\nfilters) which alter the response body:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre"},"limit:          The number of results to show. (default: 50, maximum: 250)\nsince_id:       Show only results after the specified ID.\ncreated_at_min: Show events created at or after this date and time. (format: 2014-04-25T16:15:47-04:00)\ncreated_at_max: Show events created at or before this date and time. (format: 2014-04-25T16:15:47-04:00)\nfilter:         Show events specified in this filter.\nverb:           Show events of a certain type.\nfields:         Show only certain fields, specified by a comma-separated list of field names.\n")),(0,o.kt)("p",null,"Luckily the filtering behaviour is simple, and as long as the implementation\nstays true to the description in the docs it should be easy to emulate."),(0,o.kt)("p",null,"The second endpoint takes one query parameter which is a comma-separated list of\nfields to return for a given event. Again should be easy enough."),(0,o.kt)("h3",{id:"3-authentication"},"3. Authentication"),(0,o.kt)("p",null,"We decided not to touch authentication for now as the scope would blow up for a\n5-day POC. Interestingly we got a bunch of feedback that mocking OAuth flows or\nsimilar would be ",(0,o.kt)("em",{parentName:"p"},"really")," useful, regardless of any specific API. We may come\nback to this at a future date."),(0,o.kt)("h2",{id:"day-3-evaluating-implementation-alternatives"},"Day 3: Evaluating Implementation Alternatives"),(0,o.kt)("p",null,"And then there was Day 3. We'd done our due diligence to pick a popular yet\nunderserved API, and we'd drilled down on how deep we could go in trying to\nfaithfully represent the implementation."),(0,o.kt)("p",null,"As any self-respecting engineer would do, we decided to scour the internet for\noff-the-shelf solutions to automate as much of the grunt work as possible. Some\nnaive Googling brought up a mock server called\n",(0,o.kt)("a",{parentName:"p",href:"https://github.com/typicode/json-server"},"JSON server")," - an API\nautomation solution which spins up a REST API for you given a data definition.\nExcited by this we quickly wrote up 2 fake Event API events, and\nstarted JSON server feeding it the fake events - and it worked!"),(0,o.kt)("p",null,"Well almost; we were initially excited by the fact that it did exactly what is\nsaid on the tin and very well, however it didn't have an easy way to specify the\ncustom query parameters we needed to faithfully reproduce the API. For example\nreturning results before or after a given ",(0,o.kt)("inlineCode",{parentName:"p"},"created_at")," timestamp (feel free to\nlet us know if we missed something here!)."),(0,o.kt)("p",null,"So we needed something a little more sophisticated. The internet came to the\nrescue again with a comprehensive\n",(0,o.kt)("a",{parentName:"p",href:"https://en.wikipedia.org/wiki/Comparison_of_API_simulation_tools"},"list")," of API\nsimulation tools. The basic precondition we had was that the API simulator had\nto be OSS with a permissive license we could build on. This immediately\ndisqualified 50% of the available solutions, and we did a divide and conquer\nexercise quickly evaluating the rest."),(0,o.kt)("p",null,(0,o.kt)("img",{alt:"Api Simulation Tools",src:n(8191).Z})),(0,o.kt)("p",null,"The remaining tools were either not built for this purpose, or they were\nincredibly complex pieces of software that would take a while to get acquainted\nwith."),(0,o.kt)("p",null,"In the end we decided to implement the endpoint functionality ourselves - we\nfigured that a 50 LOC node/express server would do a fine job for a POC."),(0,o.kt)("h2",{id:"day-4-implementing-the-core-functionality"},"Day 4: Implementing the core functionality"),(0,o.kt)("p",null,"Day 4 was the most straight forward. Let's get this thing to work!"),(0,o.kt)("h3",{id:"1-the-data-model-1"},"1. The Data Model"),(0,o.kt)("p",null,"We decided to reproduce the API at level 1-2 since we didn't really have any\nother endpoints. We used ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/synth"},"synth")," to quickly\nwhip up\na ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/model-repository/blob/main/shopify/shopify/events.json.json"},"data model"),"\nthat generates data that looks like responses from the Event API. I won't go\ninto depth on\nhow this works here as it's been covered\nin ",(0,o.kt)("a",{parentName:"p",href:"/blog/2021/08/31/seeding-databases-tutorial"},"other posts"),". In about 15\nminutes of tweaking the ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," schema, we generated ~10 Mb data that looks\nlike this:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-json"},'[\n    {we had\n        "arguments": "generate virtual platforms",\n        "body": null,\n        "created_at": "2019-09-17T14:16:47",\n        "description": "Received new order",\n        "id": 477306,\n        "message": "Received new order",\n        "path": "/admin/orders/83672/transactions/#6020",\n        "subject_id": 1352997,\n        "subject_type": "Order",\n        "verb": "closed"\n    },\n    {\n        "arguments": "innovate best-of-breed schemas",\n        "body": null,\n        "created_at": "2017-05-20T00:04:41",\n        "description": "Received new order",\n        "id": 370051,\n        "message": "Received new order",\n        "path": "/admin/orders/82607/transactions/#9154",\n        "subject_id": 1226112,\n        "subject_type": "Order",\n        "verb": "sale_pending"\n    },\n    {\n        "arguments": "incentivize scalable mindshare",\n        "body": null,\n        "created_at": "2018-02-21T12:51:36",\n        "description": "Received new order",\n        "id": 599084,\n        "message": "Received new order",\n        "path": "/admin/orders/01984/transactions/#3595",\n        "subject_id": 1050540,\n        "subject_type": "Order",\n        "verb": "placed"\n    }\n]\n')),(0,o.kt)("p",null,"We then dumped it all in a MongoDB collection with the one-liner:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-bash"},"$ synth generate shopify --to mongodb://localhost:27017/shopify --size 40000\n")),(0,o.kt)("p",null,"Next step is to re-create the Event API endpoint."),(0,o.kt)("h3",{id:"2-creating-the-api"},"2. Creating the API"),(0,o.kt)("p",null,"Creating the API was pretty straightforward. We wrote\na ",(0,o.kt)("a",{parentName:"p",href:"https://www.prisma.io/"},"prisma")," ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/model-repository/blob/main/shopify/prisma/schema.prisma"},"model"),"\nfor the responses which basically worked out of the box with the data dumped\ninto MongoDB by ",(0,o.kt)("inlineCode",{parentName:"p"},"synth"),". This gave us all the filtering we needed basically for\nfree."),(0,o.kt)("p",null,"Then we wrote a quick and dirty express server that maps the REST endpoint's\nquerystrings into a query for prisma. The whole thing turned out to be ~90 LOC.\nYou can check the\nsource ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/model-repository/blob/main/shopify/src/bin.ts"},"here"),"\n."),(0,o.kt)("h2",{id:"day-5-packaging"},"Day 5: Packaging"),(0,o.kt)("p",null,(0,o.kt)("img",{alt:"Docker",src:n(7866).Z})),(0,o.kt)("p",null,"The data is ready, the API is ready, time to package this thing up and give it\nto people to actually use. Let's see if our experiment was a success."),(0,o.kt)("p",null,"While strategising about distributing our API, we were optimising for two\nthings:"),(0,o.kt)("ol",null,(0,o.kt)("li",{parentName:"ol"},"Ease of use - how simple it is for someone to download this thing and get\ngoing"),(0,o.kt)("li",{parentName:"ol"},"Time - we have 2-3 hours to make sure this thing is packaged and ready to go")),(0,o.kt)("p",null,"in that order."),(0,o.kt)("p",null,"We needed to pack the data, database and a node runtime to actually run the\nserver. Our initial idea was to use ",(0,o.kt)("inlineCode",{parentName:"p"},"docker-compose")," with 2 services, the\ndatabase and web-server, and then the network plumbing to get it to work. After\ndiscussing this for a few minutes, we decided that ",(0,o.kt)("inlineCode",{parentName:"p"},"docker-compose")," may be an\noff-ramp for some users as they don't have it installed or are not familiar\nwith how it works. This went against our first tenet which is 'ease of use'."),(0,o.kt)("p",null,"So we decided to take the slightly harder and hackier route of packaging the\nwhole thing in a single Docker container. It seemed like the best trade-off\nbetween goals 1 and 2."),(0,o.kt)("p",null,"There were 6 steps to getting this thing over the line:"),(0,o.kt)("ol",null,(0,o.kt)("li",{parentName:"ol"},"Start with the MongoDB base image. This gives us a Linux environment and a\ndatabase."),(0,o.kt)("li",{parentName:"ol"},"Download and install NodeJS runtime in the container."),(0,o.kt)("li",{parentName:"ol"},"Download and install ",(0,o.kt)("inlineCode",{parentName:"li"},"synth")," in the container."),(0,o.kt)("li",{parentName:"ol"},"Copy the javascript sources over & the ",(0,o.kt)("inlineCode",{parentName:"li"},"synth")," data model"),(0,o.kt)("li",{parentName:"ol"},"Write a\nsmall ",(0,o.kt)("a",{parentName:"li",href:"https://docs.docker.com/engine/reference/builder/#entrypoint"},"ENTRYPOINT")," ",(0,o.kt)("a",{parentName:"li",href:"https://github.com/getsynth/model-repository/blob/main/shopify/start.sh"},"shell script"),"\nto start the ",(0,o.kt)("inlineCode",{parentName:"li"},"mongod"),", server and generate data into the server"),(0,o.kt)("li",{parentName:"ol"},"Start the server and expose port 3000")),(0,o.kt)("p",null,"And we're done! We've ",(0,o.kt)("del",{parentName:"p"},"hackily")," happily packaged our mock API in a platform\nagnostic one liner."),(0,o.kt)("h2",{id:"was-it-a-success"},"Was it a success?"),(0,o.kt)("p",null,"An important aspect of this experiment was to see if we could conceive,\nresearch, design and implement a PoC in a week (as a side project, we were\nworking on ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," at the same time). I can safely say this was a\nsuccess! We got it done to spec. "),(0,o.kt)("p",null,"An interesting thing to note is that ",(0,o.kt)("strong",{parentName:"p"},"60%")," of the time was spent on\nideating, researching and planning - and only 40% of the time on the actual\nimplementation. However, spending all that time planning before writing code\ndefinitely saved us a bunch of time, and if we didn't plan so much the project would have\novershot or failed."),(0,o.kt)("p",null,"Now if the PoC itself was a success is a different question. This is where ",(0,o.kt)("em",{parentName:"p"},"you"),"\ncome in. If you're using the Event API, build the image and play around with it."),(0,o.kt)("p",null,"You can get started by quickly cloning\nour ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/model-repository"},"git repository")," and then:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-bash"},"cd shopify && docker build -t shopify-mock . && docker run --rm -p 3000:3000 shopify-mock\n")),(0,o.kt)("p",null,"then simply:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-bash"},'curl "localhost:3000/admin/api/2021-07/events.json"\n')),(0,o.kt)("p",null,"We'd like to keep iterating on the Shopify API and improve it. If there is\ninterest we'll add more endpoints and improve the existing Event data model."),(0,o.kt)("p",null,"If you'd like to contribute, or are interested mocks for other APIs other than\nShopify, feel free to open an issue on GitHub!"))}u.isMDXComponent=!0},8191:function(e,t,n){t.Z=n.p+"assets/images/api-simulation-tools-c7de570989483d81b479120afad64cc8.png"},8329:function(e,t,n){t.Z=n.p+"assets/images/api-67374e47b66511f4dc2d22baa8856a8a.jpg"},7866:function(e,t,n){t.Z=n.p+"assets/images/docker-c542ea93652f9a10bce77fd9997e293a.jpg"}}]);