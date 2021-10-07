"use strict";(self.webpackChunksynth_docs=self.webpackChunksynth_docs||[]).push([[7224],{3905:function(e,n,t){t.d(n,{Zo:function(){return c},kt:function(){return d}});var r=t(7294);function a(e,n,t){return n in e?Object.defineProperty(e,n,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[n]=t,e}function i(e,n){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);n&&(r=r.filter((function(n){return Object.getOwnPropertyDescriptor(e,n).enumerable}))),t.push.apply(t,r)}return t}function s(e){for(var n=1;n<arguments.length;n++){var t=null!=arguments[n]?arguments[n]:{};n%2?i(Object(t),!0).forEach((function(n){a(e,n,t[n])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):i(Object(t)).forEach((function(n){Object.defineProperty(e,n,Object.getOwnPropertyDescriptor(t,n))}))}return e}function o(e,n){if(null==e)return{};var t,r,a=function(e,n){if(null==e)return{};var t,r,a={},i=Object.keys(e);for(r=0;r<i.length;r++)t=i[r],n.indexOf(t)>=0||(a[t]=e[t]);return a}(e,n);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(r=0;r<i.length;r++)t=i[r],n.indexOf(t)>=0||Object.prototype.propertyIsEnumerable.call(e,t)&&(a[t]=e[t])}return a}var l=r.createContext({}),p=function(e){var n=r.useContext(l),t=n;return e&&(t="function"==typeof e?e(n):s(s({},n),e)),t},c=function(e){var n=p(e.components);return r.createElement(l.Provider,{value:n},e.children)},m={inlineCode:"code",wrapper:function(e){var n=e.children;return r.createElement(r.Fragment,{},n)}},u=r.forwardRef((function(e,n){var t=e.components,a=e.mdxType,i=e.originalType,l=e.parentName,c=o(e,["components","mdxType","originalType","parentName"]),u=p(t),d=a,h=u["".concat(l,".").concat(d)]||u[d]||m[d]||i;return t?r.createElement(h,s(s({ref:n},c),{},{components:t})):r.createElement(h,s({ref:n},c))}));function d(e,n){var t=arguments,a=n&&n.mdxType;if("string"==typeof e||a){var i=t.length,s=new Array(i);s[0]=u;var o={};for(var l in n)hasOwnProperty.call(n,l)&&(o[l]=n[l]);o.originalType=e,o.mdxType="string"==typeof e?e:a,s[1]=o;for(var p=2;p<i;p++)s[p]=t[p];return r.createElement.apply(null,s)}return r.createElement.apply(null,t)}u.displayName="MDXCreateElement"},6632:function(e,n,t){t.r(n),t.d(n,{frontMatter:function(){return o},contentTitle:function(){return l},metadata:function(){return p},toc:function(){return c},default:function(){return u}});var r=t(7462),a=t(3366),i=(t(7294),t(3905)),s=["components"],o={},l=void 0,p={unversionedId:"docs/content/series",id:"docs/content/series",isDocsHomePage:!1,title:"series",description:"Synth's series generator creates streams of events based on different 'processes' (a process here can be an auto-correlated process, a poisson process, a cyclical process etc.).",source:"@site/docs/docs/content/series.md",sourceDirName:"docs/content",slug:"/docs/content/series",permalink:"/docs/content/series",editUrl:"https://github.com/getsynth/synth/edit/master/docs/docs/docs/content/series.md",tags:[],version:"current",frontMatter:{},sidebar:"docsSidebar",previous:{title:"unique",permalink:"/docs/content/unique"},next:{title:"Telemetry",permalink:"/other/telemetry"}},c=[{value:"incrementing",id:"incrementing",children:[]},{value:"poisson",id:"poisson",children:[]},{value:"cyclical",id:"cyclical",children:[]},{value:"zip",id:"zip",children:[]}],m={toc:c};function u(e){var n=e.components,t=(0,a.Z)(e,s);return(0,i.kt)("wrapper",(0,r.Z)({},m,t,{components:n,mdxType:"MDXLayout"}),(0,i.kt)("p",null,"Synth's ",(0,i.kt)("inlineCode",{parentName:"p"},"series")," generator creates streams of events based on different 'processes' (a process here can be an auto-correlated process, a poisson process, a cyclical process etc.)."),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"series")," generators are used in several different contexts:"),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},"Creating fake events for event-driven systems"),(0,i.kt)("li",{parentName:"ul"},"Modelling time-independent events like 'orders' on a website or 'requests' made to a web server"),(0,i.kt)("li",{parentName:"ul"},"Modelling seasonal behaviour, like an increase in flight frequency for a given airline over the summer")),(0,i.kt)("h4",{id:"date-time"},"Date Time"),(0,i.kt)("p",null,"All ",(0,i.kt)("inlineCode",{parentName:"p"},"series")," are modelled on so called 'Naive Date Times' - that is 'Date Times' that do not have a timezone. This can be interpreted as Timestamps in UTC. There is future work to improve functionality to add other chrono types."),(0,i.kt)("p",null,"The format of a series can be set by using the optional ",(0,i.kt)("inlineCode",{parentName:"p"},"format")," field; if ",(0,i.kt)("inlineCode",{parentName:"p"},"format")," is omitted, the default format is ",(0,i.kt)("inlineCode",{parentName:"p"},"%Y-%m-%d %H:%M:%S"),"."),(0,i.kt)("h4",{id:"duration"},"Duration"),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"series")," generators will often make use of durations as generation parameters. A duration as a quantity like '1 hour' or '5.7 milliseconds'."),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"series")," generators use ",(0,i.kt)("a",{parentName:"p",href:"https://docs.rs/humantime/2.1.0/humantime/fn.parse_duration.html"},(0,i.kt)("inlineCode",{parentName:"a"},"humantime"))," to make it easy to specify human readable quantities like ",(0,i.kt)("inlineCode",{parentName:"p"},"3hr 5m 2s"),"."),(0,i.kt)("h2",{id:"incrementing"},"incrementing"),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"incrementing")," series simply increments at a fixed duration. This could be for example a stock ticker."),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"incrementing")," series has 2 parameters:"),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"start"),": The time at which the first event occurs"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"increment"),": The increment between two consecutive events")),(0,i.kt)("h4",{id:"example"},"Example"),(0,i.kt)("p",null,"Below is an example stock ticker for AAPL sampled at regular intervals every minute. "),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "constant": 10\n    },\n    "content": {\n        "type": "object",\n        "ticker": {\n            "type": "string",\n            "pattern": "AAPL"\n        },\n        "timestamp": {\n            "type": "series",\n            "format" : "%Y-%m-%d %H:%M:%S",\n            "incrementing": {\n                "start" : "2021-02-01 09:00:00",\n                "increment" : "1m"\n            }\n        },\n        "price": {\n            "type": "number",\n            "subtype" : "f64",\n            "range" : {\n                "high": 105, \n                "low": 100,\n                "step": 0.01\n            }\n        }\n    }\n}\n')),(0,i.kt)("h2",{id:"poisson"},"poisson"),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"poisson")," series models independent events which occur at random, but which tend to occur at an average rate when viewed as a group."),(0,i.kt)("p",null,"One example of a poisson process could be earthquakes occurring during the course of a year, or customers arriving at a store, or cars crossing a bridge etc."),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"poisson")," series has 2 parameters:"),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"start"),": The time at which the first event occurs"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"rate"),": The average duration between two consecutive events")),(0,i.kt)("h4",{id:"example-1"},"Example"),(0,i.kt)("p",null,"The below is an example HTTP server, which was brought up on a given date and has an average of 1 request every 1 minute."),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "constant": 10\n    },\n    "content": {\n        "type": "object",\n        "ip": {\n            "type": "string",\n            "faker": {\n                "generator": "ipv4"\n            }\n        },\n        "timestamp": {\n            "type": "series",\n            "format": "%d/%b/%Y:%H:%M:%S",\n            "poisson": {\n                "start": "10/Oct/2000:13:55:36",\n                "rate": "1m"\n            }\n        },\n        "request": {\n            "type": "string",\n            "categorical": {\n                "GET /index.html HTTP/1.0": 10,\n                "GET /home.html HTTP/1.0": 5,\n                "GET /login.html HTTP/1.0": 3\n            }\n        },\n        "response_code": {\n            "type": "number",\n            "subtype": "u64",\n            "categorical": {\n                "200": 95,\n                "500": 5\n            }\n        },\n        "response_size": {\n            "type": "number",\n            "range": {\n                "low": 500,\n                "high": 3000,\n                "step": 1\n            }\n        }\n    }\n}\n')),(0,i.kt)("h2",{id:"cyclical"},"cyclical"),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"cyclical")," series models events which have a 'cyclical' or 'periodic' frequency. "),(0,i.kt)("p",null,"For example, the frequency of orders placed in an online store peaks during the day and is at it's lowest during the night."),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"cyclical")," series has 4 parameters:"),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"start"),": The time at which the first event occurs"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"max_rate"),": The maximum average duration between two events."),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"min_rate"),": The minimum average duration between two events"),(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"period"),": The period of the cyclical series.")),(0,i.kt)("h4",{id:"example-2"},"Example"),(0,i.kt)("p",null,"The below is a minimal example of orders being placed in an online store."),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "constant": 10\n    },\n    "content": {\n        "type": "object",\n        "order_id": {\n            "type": "number",\n            "id": {}\n        },\n        "item": {\n            "type": "string",\n            "categorical": {\n                "t-shirt": 4,\n                "jeans": 1,\n                "jacket": 1,\n                "belt": 2\n            }\n        },\n        "timestamp": {\n            "type": "series",\n            "cyclical": {\n                "start": "2021-02-01 00:00:00",\n                "period": "1d",\n                "min_rate": "10m",\n                "max_rate": "30s"\n            }\n        }\n    }\n}\n')),(0,i.kt)("h2",{id:"zip"},"zip"),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"zip")," series combines 2 or more series together by ",(0,i.kt)("inlineCode",{parentName:"p"},"zipping")," the output together. That is, the two series are super imposed."),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"zip")," series has 1 parameter:"),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},(0,i.kt)("inlineCode",{parentName:"li"},"series"),": The child series to be zipped together")),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "constant": 10\n    },\n    "content": {\n        "type": "object",\n        "order_id": {\n            "type": "number",\n            "id": {}\n        },\n        "item": {\n            "type": "string",\n            "categorical": {\n                "t-shirt": 4,\n                "jeans": 1,\n                "jacket": 1,\n                "belt": 2\n            }\n        },\n        "timestamp": {\n            "type": "series",\n            "zip": {\n                "series": [\n                    {\n                        "cyclical": {\n                            "start": "2021-02-01 00:00:00",\n                            "period": "1w",\n                            "min_rate": "1m",\n                            "max_rate": "1s"\n                        }\n                    },\n                    {\n                        "cyclical": {\n                            "start": "2021-02-01 00:00:00",\n                            "period": "1d",\n                            "min_rate": "10m",\n                            "max_rate": "30s"\n                        }\n                    }\n                ]\n            }\n        }\n    }\n}\n')))}u.isMDXComponent=!0}}]);