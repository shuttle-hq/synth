"use strict";(self.webpackChunksynth_docs=self.webpackChunksynth_docs||[]).push([[3668],{3905:function(e,t,n){n.d(t,{Zo:function(){return m},kt:function(){return u}});var a=n(7294);function r(e,t,n){return t in e?Object.defineProperty(e,t,{value:n,enumerable:!0,configurable:!0,writable:!0}):e[t]=n,e}function o(e,t){var n=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);t&&(a=a.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),n.push.apply(n,a)}return n}function i(e){for(var t=1;t<arguments.length;t++){var n=null!=arguments[t]?arguments[t]:{};t%2?o(Object(n),!0).forEach((function(t){r(e,t,n[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(n)):o(Object(n)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(n,t))}))}return e}function l(e,t){if(null==e)return{};var n,a,r=function(e,t){if(null==e)return{};var n,a,r={},o=Object.keys(e);for(a=0;a<o.length;a++)n=o[a],t.indexOf(n)>=0||(r[n]=e[n]);return r}(e,t);if(Object.getOwnPropertySymbols){var o=Object.getOwnPropertySymbols(e);for(a=0;a<o.length;a++)n=o[a],t.indexOf(n)>=0||Object.prototype.propertyIsEnumerable.call(e,n)&&(r[n]=e[n])}return r}var p=a.createContext({}),s=function(e){var t=a.useContext(p),n=t;return e&&(n="function"==typeof e?e(t):i(i({},t),e)),n},m=function(e){var t=s(e.components);return a.createElement(p.Provider,{value:t},e.children)},d={inlineCode:"code",wrapper:function(e){var t=e.children;return a.createElement(a.Fragment,{},t)}},c=a.forwardRef((function(e,t){var n=e.components,r=e.mdxType,o=e.originalType,p=e.parentName,m=l(e,["components","mdxType","originalType","parentName"]),c=s(n),u=r,g=c["".concat(p,".").concat(u)]||c[u]||d[u]||o;return n?a.createElement(g,i(i({ref:t},m),{},{components:n})):a.createElement(g,i({ref:t},m))}));function u(e,t){var n=arguments,r=t&&t.mdxType;if("string"==typeof e||r){var o=n.length,i=new Array(o);i[0]=c;var l={};for(var p in t)hasOwnProperty.call(t,p)&&(l[p]=t[p]);l.originalType=e,l.mdxType="string"==typeof e?e:r,i[1]=l;for(var s=2;s<o;s++)i[s]=n[s];return a.createElement.apply(null,i)}return a.createElement.apply(null,n)}c.displayName="MDXCreateElement"},3409:function(e,t,n){n.r(t),n.d(t,{frontMatter:function(){return l},contentTitle:function(){return p},metadata:function(){return s},toc:function(){return m},default:function(){return c}});var a=n(7462),r=n(3366),o=(n(7294),n(3905)),i=["components"],l={title:"PostgreSQL"},p=void 0,s={unversionedId:"docs/integrations/postgres",id:"docs/integrations/postgres",isDocsHomePage:!1,title:"PostgreSQL",description:"The Synth PostgreSQL integration is currently in beta.",source:"@site/docs/docs/integrations/postgres.md",sourceDirName:"docs/integrations",slug:"/docs/integrations/postgres",permalink:"/docs/integrations/postgres",editUrl:"https://github.com/getsynth/synth/edit/master/docs/docs/docs/integrations/postgres.md",tags:[],version:"current",frontMatter:{title:"PostgreSQL"},sidebar:"docsSidebar",previous:{title:"bank_db",permalink:"/docs/examples/bank"},next:{title:"All generators",permalink:"/docs/content/index"}},m=[{value:"Usage",id:"usage",children:[{value:"URI format",id:"uri-format",children:[]}]},{value:"Import",id:"import",children:[{value:"Example Import",id:"example-import",children:[]},{value:"Example Import Command",id:"example-import-command",children:[]},{value:"Example",id:"example",children:[]}]},{value:"Generate",id:"generate",children:[{value:"Example Generation Command",id:"example-generation-command",children:[]}]}],d={toc:m};function c(e){var t=e.components,n=(0,r.Z)(e,i);return(0,o.kt)("wrapper",(0,a.Z)({},d,n,{components:t,mdxType:"MDXLayout"}),(0,o.kt)("div",{className:"admonition admonition-note alert alert--secondary"},(0,o.kt)("div",{parentName:"div",className:"admonition-heading"},(0,o.kt)("h5",{parentName:"div"},(0,o.kt)("span",{parentName:"h5",className:"admonition-icon"},(0,o.kt)("svg",{parentName:"span",xmlns:"http://www.w3.org/2000/svg",width:"14",height:"16",viewBox:"0 0 14 16"},(0,o.kt)("path",{parentName:"svg",fillRule:"evenodd",d:"M6.3 5.69a.942.942 0 0 1-.28-.7c0-.28.09-.52.28-.7.19-.18.42-.28.7-.28.28 0 .52.09.7.28.18.19.28.42.28.7 0 .28-.09.52-.28.7a1 1 0 0 1-.7.3c-.28 0-.52-.11-.7-.3zM8 7.99c-.02-.25-.11-.48-.31-.69-.2-.19-.42-.3-.69-.31H6c-.27.02-.48.13-.69.31-.2.2-.3.44-.31.69h1v3c.02.27.11.5.31.69.2.2.42.31.69.31h1c.27 0 .48-.11.69-.31.2-.19.3-.42.31-.69H8V7.98v.01zM7 2.3c-3.14 0-5.7 2.54-5.7 5.68 0 3.14 2.56 5.7 5.7 5.7s5.7-2.55 5.7-5.7c0-3.15-2.56-5.69-5.7-5.69v.01zM7 .98c3.86 0 7 3.14 7 7s-3.14 7-7 7-7-3.12-7-7 3.14-7 7-7z"}))),"note")),(0,o.kt)("div",{parentName:"div",className:"admonition-content"},(0,o.kt)("p",{parentName:"div"},"The Synth PostgreSQL integration is currently ",(0,o.kt)("strong",{parentName:"p"},"in beta"),"."))),(0,o.kt)("h2",{id:"usage"},"Usage"),(0,o.kt)("p",null,(0,o.kt)("inlineCode",{parentName:"p"},"synth")," can use ",(0,o.kt)("a",{parentName:"p",href:"https://www.postgresql.org/"},"PostgreSQL")," as a data source or\nsink. Connecting ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," to a PostgreSQL is as simple as specifying a URI\nand schema during the ",(0,o.kt)("inlineCode",{parentName:"p"},"import")," or ",(0,o.kt)("inlineCode",{parentName:"p"},"generate"),"\nphase."),(0,o.kt)("h3",{id:"uri-format"},"URI format"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-bash"},"postgres://<username>:<password>@<host>:<port>/<catalog>\n")),(0,o.kt)("h2",{id:"import"},"Import"),(0,o.kt)("p",null,(0,o.kt)("inlineCode",{parentName:"p"},"synth")," can import directly from a ",(0,o.kt)("a",{parentName:"p",href:"https://www.postgresql.org/"},"PostgreSQL"),"\ndatabase and create a data model from the database schema. During import, a\nnew ",(0,o.kt)("a",{parentName:"p",href:"../getting_started/core-concepts#namespaces"},"namespace"),"\nwill be created from your database schema, and\na ",(0,o.kt)("a",{parentName:"p",href:"../getting_started/core-concepts#collections"},"collection")," is created for each\ntable in a separate JSON file. ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," will map database columns to fields in\nthe collections it creates. It then provides default generators for every\ncollection. Synth will default to the ",(0,o.kt)("inlineCode",{parentName:"p"},"public")," schema but this can be\noverriden with the ",(0,o.kt)("inlineCode",{parentName:"p"},"--schema")," flag."),(0,o.kt)("p",null,(0,o.kt)("inlineCode",{parentName:"p"},"synth")," will automatically detect primary key and foreign key constraints at\nimport time and update the namespace and collection to reflect them. ",(0,o.kt)("strong",{parentName:"p"},"Primary\nkeys")," get mapped to ",(0,o.kt)("inlineCode",{parentName:"p"},"synth"),"'s ",(0,o.kt)("a",{parentName:"p",href:"../content/number#id"},"id"),"\ngenerator, and ",(0,o.kt)("strong",{parentName:"p"},"foreign keys")," get mapped to the ",(0,o.kt)("a",{parentName:"p",href:"/docs/content/same-as"},"same_as"),"\ngenerator."),(0,o.kt)("p",null,"Finally ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," will sample data randomly from every table in order to create a\nmore realistic data model by automatically inferring bounds on types."),(0,o.kt)("p",null,(0,o.kt)("inlineCode",{parentName:"p"},"synth")," has its own internal data model, and so does Postgres, therefore a\nconversion occurs between ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," types and Postgres types. The inferred type\ncan be seen below. The synth types link to default generator ",(0,o.kt)("em",{parentName:"p"},"variant"),"\ngenerated during the ",(0,o.kt)("inlineCode",{parentName:"p"},"import")," process for that PostgreSQL type."),(0,o.kt)("p",null,"Note, not all PostgreSQL types have been covered yet. If there is a type you\nneed, ",(0,o.kt)("a",{parentName:"p",href:"https://github.com/getsynth/synth/issues/new?assignees=&labels=New+feature&template=feature_request.md&title="},"open an issue"),"\non GitHub."),(0,o.kt)("table",null,(0,o.kt)("thead",{parentName:"table"},(0,o.kt)("tr",{parentName:"thead"},(0,o.kt)("th",{parentName:"tr",align:null},"PostgreSQL Type"),(0,o.kt)("th",{parentName:"tr",align:null},"Synth Type"))),(0,o.kt)("tbody",{parentName:"table"},(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"Null ","|"," T"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/one-of"},"one_of"),"<",(0,o.kt)("a",{parentName:"td",href:"../content/null"},"null"),", T>")),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"boolean"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/bool#frequency"},"bool"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"char"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#pattern"},"string"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"varchar(x)"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#pattern"},"string"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"text"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#pattern"},"string"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"bpchar(x)"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#pattern"},"string"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"name"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#pattern"},"string"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"int2"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/number#range"},"i64"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"int4"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/number#range"},"i32"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"int8"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/number#range"},"i64"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"float4"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/number#range"},"f32"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"float8"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/number#range"},"f64"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"numeric"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/number#range"},"f64"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"timestamptz"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#date_time"},"date_time"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"timestamp"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#date_time"},"naive_date_time"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"date"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#date_time"},"naive_date"))),(0,o.kt)("tr",{parentName:"tbody"},(0,o.kt)("td",{parentName:"tr",align:null},"uuid"),(0,o.kt)("td",{parentName:"tr",align:null},(0,o.kt)("a",{parentName:"td",href:"../content/string#uuid"},"string"))))),(0,o.kt)("h3",{id:"example-import"},"Example Import"),(0,o.kt)("p",null,"Below is an example import for a single table."),(0,o.kt)("p",null,"Postgres table definition:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-sql"},"create table doctors\n(\n    id          int primary key,\n    hospital_id int not null, \n    name        varchar(255) not null,\n    date_joined date,\n    constraint hospital_fk\n        foreign key(hospital_id)\n            references hospitals(id)\n);\n")),(0,o.kt)("p",null,"And the corresponding ",(0,o.kt)("inlineCode",{parentName:"p"},"synth")," collection:"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-json"},'{\n  "type": "array",\n  "length": {\n    "type": "number",\n    "range": {\n      "low": 0,\n      "high": 2,\n      "step": 1\n    },\n    "subtype": "u64"\n  },\n  "content": {\n    "type": "object",\n    "date_joined": {\n      "type": "one_of",\n      "variants": [\n        {\n          "weight": 1.0,\n          "type": "string",\n          "date_time": {\n            "format": "%Y-%m-%d",\n            "subtype": "naive_date",\n            "begin": null,\n            "end": null\n          }\n        },\n        {\n          "weight": 1.0,\n          "type": "null"\n        }\n      ]\n    },\n    "hospital_id": {\n      "type": "same_as",\n      "ref": "hospitals.content.id"\n    },\n    "id": {\n      "type": "number",\n      "id": {},\n      "subtype": "u64"\n    },\n    "name": {\n      "type": "string",\n      "pattern": "[a-zA-Z0-9]{0, 255}"\n    }\n  }\n\n')),(0,o.kt)("h3",{id:"example-import-command"},"Example Import Command"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-bash"},"synth import --from postgres://user:pass@localhost:5432/postgres --schema \nmain my_namespace \n")),(0,o.kt)("h3",{id:"example"},"Example"),(0,o.kt)("h2",{id:"generate"},"Generate"),(0,o.kt)("p",null,(0,o.kt)("inlineCode",{parentName:"p"},"synth")," can generate data directly into your PostgreSQL database. First ",(0,o.kt)("inlineCode",{parentName:"p"},"synth"),"\nwill generate as much data as required, then open a connection to your database,\nand then perform batch insert to quickly insert as much data as you need."),(0,o.kt)("p",null,(0,o.kt)("inlineCode",{parentName:"p"},"synth")," will also respect primary key and foreign key constraints, by performing\na ",(0,o.kt)("a",{parentName:"p",href:"https://en.wikipedia.org/wiki/Topological_sorting"},"topological sort")," on the\ndata and inserting it in the right order such that no constraints are violated."),(0,o.kt)("h3",{id:"example-generation-command"},"Example Generation Command"),(0,o.kt)("pre",null,(0,o.kt)("code",{parentName:"pre",className:"language-bash"},"synth generate --to postgres://user:pass@localhost:5432/ --schema \nmain my_namespace\n")))}c.isMDXComponent=!0}}]);