"use strict";(self.webpackChunksynth_docs=self.webpackChunksynth_docs||[]).push([[7469],{3905:function(e,n,t){t.d(n,{Zo:function(){return p},kt:function(){return h}});var a=t(7294);function r(e,n,t){return n in e?Object.defineProperty(e,n,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[n]=t,e}function i(e,n){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);n&&(a=a.filter((function(n){return Object.getOwnPropertyDescriptor(e,n).enumerable}))),t.push.apply(t,a)}return t}function o(e){for(var n=1;n<arguments.length;n++){var t=null!=arguments[n]?arguments[n]:{};n%2?i(Object(t),!0).forEach((function(n){r(e,n,t[n])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):i(Object(t)).forEach((function(n){Object.defineProperty(e,n,Object.getOwnPropertyDescriptor(t,n))}))}return e}function s(e,n){if(null==e)return{};var t,a,r=function(e,n){if(null==e)return{};var t,a,r={},i=Object.keys(e);for(a=0;a<i.length;a++)t=i[a],n.indexOf(t)>=0||(r[t]=e[t]);return r}(e,n);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(a=0;a<i.length;a++)t=i[a],n.indexOf(t)>=0||Object.prototype.propertyIsEnumerable.call(e,t)&&(r[t]=e[t])}return r}var l=a.createContext({}),c=function(e){var n=a.useContext(l),t=n;return e&&(t="function"==typeof e?e(n):o(o({},n),e)),t},p=function(e){var n=c(e.components);return a.createElement(l.Provider,{value:n},e.children)},m={inlineCode:"code",wrapper:function(e){var n=e.children;return a.createElement(a.Fragment,{},n)}},u=a.forwardRef((function(e,n){var t=e.components,r=e.mdxType,i=e.originalType,l=e.parentName,p=s(e,["components","mdxType","originalType","parentName"]),u=c(t),h=r,d=u["".concat(l,".").concat(h)]||u[h]||m[h]||i;return t?a.createElement(d,o(o({ref:n},p),{},{components:t})):a.createElement(d,o({ref:n},p))}));function h(e,n){var t=arguments,r=n&&n.mdxType;if("string"==typeof e||r){var i=t.length,o=new Array(i);o[0]=u;var s={};for(var l in n)hasOwnProperty.call(n,l)&&(s[l]=n[l]);s.originalType=e,s.mdxType="string"==typeof e?e:r,o[1]=s;for(var c=2;c<i;c++)o[c]=t[c];return a.createElement.apply(null,o)}return a.createElement.apply(null,t)}u.displayName="MDXCreateElement"},6635:function(e,n,t){t.r(n),t.d(n,{frontMatter:function(){return s},contentTitle:function(){return l},metadata:function(){return c},toc:function(){return p},default:function(){return u}});var a=t(7462),r=t(3366),i=(t(7294),t(3905)),o=["components"],s={title:"Schema"},l=void 0,c={unversionedId:"docs/getting_started/schema",id:"docs/getting_started/schema",isDocsHomePage:!1,title:"Schema",description:"The schema is the core data structure that you need to understand to become a Synth wizard. Schemas are JSON files",source:"@site/docs/docs/getting_started/schema.md",sourceDirName:"docs/getting_started",slug:"/docs/getting_started/schema",permalink:"/docs/getting_started/schema",editUrl:"https://github.com/getsynth/synth/edit/master/docs/docs/docs/getting_started/schema.md",tags:[],version:"current",frontMatter:{title:"Schema"},sidebar:"docsSidebar",previous:{title:"Core concepts",permalink:"/docs/getting_started/core-concepts"},next:{title:"Command-line",permalink:"/docs/getting_started/command-line"}},p=[{value:"JSON",id:"json",children:[]},{value:"Synth Schema Nodes",id:"synth-schema-nodes",children:[]},{value:"Writing Synth Schemas",id:"writing-synth-schemas",children:[]},{value:"A real life example",id:"a-real-life-example",children:[]},{value:"What&#39;s next",id:"whats-next",children:[]}],m={toc:p};function u(e){var n=e.components,s=(0,r.Z)(e,o);return(0,i.kt)("wrapper",(0,a.Z)({},m,s,{components:n,mdxType:"MDXLayout"}),(0,i.kt)("p",null,"The ",(0,i.kt)("inlineCode",{parentName:"p"},"schema")," is the core data structure that you need to understand to become a Synth wizard. Schemas are JSON files\nthat encode the shape of the data you want to generate. All schemas are composed of ",(0,i.kt)("inlineCode",{parentName:"p"},"generators")," that are assembled by\nthe user to create complex data structures."),(0,i.kt)("p",null,"It's a little involved, so let's start with a simpler example: JSON!"),(0,i.kt)("h3",{id:"json"},"JSON"),(0,i.kt)("p",null,"If you've never actually seen how JSON is implemented under the hood, you may find this interesting."),(0,i.kt)("p",null,"One of the reasons for JSON's popularity is just how simple of a data structure it is. JSON is a recursive data\nstructure (just a tree but let's pretend we're smart) and can be defined in 8 lines of code (if you're wondering, this\nis Rust's ",(0,i.kt)("inlineCode",{parentName:"p"},"enum")," notation):"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-rust"},'enum Value {\n    Null,                       // null\n    Bool(bool),                 // true\n    Number(Number),             // 42\n    String(String),             // "Synth"\n    Array(Vec<Value>),          // [0, true, "a", ...]\n    Object(Map<String, Value>), // { "name" : "Cynthia", "age" : 42 }\n}\n')),(0,i.kt)("p",null,"So every node in a JSON tree, is one of 6 variants. Recursion occurs where ",(0,i.kt)("inlineCode",{parentName:"p"},"Array"),"s and ",(0,i.kt)("inlineCode",{parentName:"p"},"Object"),"s can have children\nwhich are also one of 6 variants."),(0,i.kt)("p",null,"We've based the Synth schema on the same design. But, what does this look like when you need to capture far more\ncomplexity than the JSON schema?"),(0,i.kt)("h3",{id:"synth-schema-nodes"},"Synth Schema Nodes"),(0,i.kt)("p",null,"Much like the ",(0,i.kt)("inlineCode",{parentName:"p"},"Value")," node in a JSON tree, the ",(0,i.kt)("inlineCode",{parentName:"p"},"Schema")," nodes in the synth Schema give us the recursive data structure\nwhich Synth can use to generate data."),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-rust"},"enum Schema {\n    Null,\n    Bool(BoolSchema),\n    Number(NumberSchema),\n    String(StringSchema), // here\n    Array(ArraySchema),\n    Object(ObjectSchema),\n    SameAs(SameAsSchema),\n    OneOf(OneOfSchema),\n}\n")),(0,i.kt)("p",null,"Each of these ",(0,i.kt)("inlineCode",{parentName:"p"},"Schema")," variants, cover a bunch of different types of ",(0,i.kt)("inlineCode",{parentName:"p"},"Schema")," nodes, just to give an example,\nthe ",(0,i.kt)("inlineCode",{parentName:"p"},"StringSchema")," variant looks like this under the hood:"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-rust"},"enum StringSchema {\n    Pattern(RegexSchema),\n    DateTime(DateTimeSchema),\n    Categorical(Categorical<String>),\n    Faker(FakerSchema),\n}\n")),(0,i.kt)("p",null,"Where ",(0,i.kt)("inlineCode",{parentName:"p"},"String")," types can be generated from regular expressions, date time generators and so on. For a comprehensive list\nsee the ",(0,i.kt)("a",{parentName:"p",href:"/docs/content/string"},"String")," docs."),(0,i.kt)("h3",{id:"writing-synth-schemas"},"Writing Synth Schemas"),(0,i.kt)("p",null,(0,i.kt)("inlineCode",{parentName:"p"},"Schema")," nodes have different fields depending on the type of node. This makes sense, if you are generating Id's,\nyou're going to want to specify different parameters to if you are generating a date or a time."),(0,i.kt)("p",null,"However, all ",(0,i.kt)("inlineCode",{parentName:"p"},"Schema")," nodes follow a similar template."),(0,i.kt)("ul",null,(0,i.kt)("li",{parentName:"ul"},"There is a boolean ",(0,i.kt)("inlineCode",{parentName:"li"},"optional")," field, which tells Synth if a field is nullable or not."),(0,i.kt)("li",{parentName:"ul"},"Next there is a ",(0,i.kt)("inlineCode",{parentName:"li"},"type")," field, which specifies which top-level ",(0,i.kt)("inlineCode",{parentName:"li"},"Schema")," type the node is (",(0,i.kt)("inlineCode",{parentName:"li"},"string"),", ",(0,i.kt)("inlineCode",{parentName:"li"},"number"),", ",(0,i.kt)("inlineCode",{parentName:"li"},"bool"),"\netc.). Fields can often have a ",(0,i.kt)("inlineCode",{parentName:"li"},"subtype")," which disambiguates certain types (for example is a ",(0,i.kt)("inlineCode",{parentName:"li"},"number")," a float ",(0,i.kt)("inlineCode",{parentName:"li"},"f64")," or\nan unsigned integer ",(0,i.kt)("inlineCode",{parentName:"li"},"u64"),".)"),(0,i.kt)("li",{parentName:"ul"},"Finally, ",(0,i.kt)("inlineCode",{parentName:"li"},"Schema")," nodes can have 0 or more fields which are specific to that node type. For more information refer to\nthe documentation for that type.")),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "number",\n  "subtype": "f64",\n  "range": {\n    "low": 274.4,\n    "high": 6597.5,\n    "step": 0.1\n  }\n}\n')),(0,i.kt)("h3",{id:"a-real-life-example"},"A real life example"),(0,i.kt)("p",null,"In our example schema we have a namespace ",(0,i.kt)("inlineCode",{parentName:"p"},"my_app")," which has 2 collections - ",(0,i.kt)("inlineCode",{parentName:"p"},"transactions")," and ",(0,i.kt)("inlineCode",{parentName:"p"},"users"),"."),(0,i.kt)("p",null,"Below is a tree representation of the schema Schema tree:"),(0,i.kt)("p",null,(0,i.kt)("img",{alt:"An example schema",src:t(3928).Z})),(0,i.kt)("p",null,"The corresponding namespace can be broken into 2 collections. The first, ",(0,i.kt)("inlineCode",{parentName:"p"},"transactions"),":"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json"},'{\n  "type": "array",\n  "length": {\n    "type": "number",\n    "subtype": "u64",\n    "range": {\n      "low": 1,\n      "high": 6,\n      "step": 1\n    }\n  },\n  "content": {\n    "type": "object",\n    "amount": {\n      "optional": false,\n      "type": "number",\n      "subtype": "f64",\n      "range": {\n        "low": 0,\n        "high": 1000,\n        "step": 0.01\n      }\n    },\n    "currency": {\n      "type": "one_of",\n      "variants": [\n        {\n          "type": "string",\n          "pattern": "USD"\n        },\n        {\n          "type": "string",\n          "pattern": "GBP"\n        }\n      ]\n    },\n    "timestamp": {\n      "type": "string",\n      "date_time": {\n        "format": "%Y-%m-%dT%H:%M:%S%z",\n        "begin": "2000-01-01T00:00:00+0000",\n        "end": "2020-01-01T00:00:00+0000"\n      }\n    },\n    "user_id": {\n      "type": "same_as",\n      "ref": "users.Schema.user_id"\n    }\n  }\n}\n')),(0,i.kt)("p",null,"And the second, the ",(0,i.kt)("inlineCode",{parentName:"p"},"users")," collection:"),(0,i.kt)("pre",null,(0,i.kt)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "subtype": "u64",\n        "range": {\n            "low": 1,\n            "high": 6,\n            "step": 1\n        }\n    },\n    "content": {\n        "type": "object",\n        "user_id": {\n            "type": "number",\n            "subtype": "u64",\n            "id": {\n                "start_at": 0\n            }\n        },\n        "user_email": {\n            "type": "string",\n            "faker": {\n                "generator": "email"\n            }\n        }\n    }\n}\n')),(0,i.kt)("h3",{id:"whats-next"},"What's next"),(0,i.kt)("p",null,"The ",(0,i.kt)("a",{parentName:"p",href:"/docs/content/null"},"generators reference")," in this documentation is the best place to become familiar with all the\ndifferent variants of schema nodes. This will let you write schemas for any of the data you might need."))}u.isMDXComponent=!0},3928:function(e,n,t){n.Z=t.p+"assets/images/schema_overview-a671cd84cab723994cad92ec6fd2b3d3.png"}}]);