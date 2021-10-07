"use strict";(self.webpackChunksynth_docs=self.webpackChunksynth_docs||[]).push([[255],{3905:function(e,t,a){a.d(t,{Zo:function(){return d},kt:function(){return c}});var n=a(7294);function r(e,t,a){return t in e?Object.defineProperty(e,t,{value:a,enumerable:!0,configurable:!0,writable:!0}):e[t]=a,e}function l(e,t){var a=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),a.push.apply(a,n)}return a}function o(e){for(var t=1;t<arguments.length;t++){var a=null!=arguments[t]?arguments[t]:{};t%2?l(Object(a),!0).forEach((function(t){r(e,t,a[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(a)):l(Object(a)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(a,t))}))}return e}function i(e,t){if(null==e)return{};var a,n,r=function(e,t){if(null==e)return{};var a,n,r={},l=Object.keys(e);for(n=0;n<l.length;n++)a=l[n],t.indexOf(a)>=0||(r[a]=e[a]);return r}(e,t);if(Object.getOwnPropertySymbols){var l=Object.getOwnPropertySymbols(e);for(n=0;n<l.length;n++)a=l[n],t.indexOf(a)>=0||Object.prototype.propertyIsEnumerable.call(e,a)&&(r[a]=e[a])}return r}var s=n.createContext({}),p=function(e){var t=n.useContext(s),a=t;return e&&(a="function"==typeof e?e(t):o(o({},t),e)),a},d=function(e){var t=p(e.components);return n.createElement(s.Provider,{value:t},e.children)},m={inlineCode:"code",wrapper:function(e){var t=e.children;return n.createElement(n.Fragment,{},t)}},u=n.forwardRef((function(e,t){var a=e.components,r=e.mdxType,l=e.originalType,s=e.parentName,d=i(e,["components","mdxType","originalType","parentName"]),u=p(a),c=r,g=u["".concat(s,".").concat(c)]||u[c]||m[c]||l;return a?n.createElement(g,o(o({ref:t},d),{},{components:a})):n.createElement(g,o({ref:t},d))}));function c(e,t){var a=arguments,r=t&&t.mdxType;if("string"==typeof e||r){var l=a.length,o=new Array(l);o[0]=u;var i={};for(var s in t)hasOwnProperty.call(t,s)&&(i[s]=t[s]);i.originalType=e,i.mdxType="string"==typeof e?e:r,o[1]=i;for(var p=2;p<l;p++)o[p]=a[p];return n.createElement.apply(null,o)}return n.createElement.apply(null,a)}u.displayName="MDXCreateElement"},3338:function(e,t,a){a.r(t),a.d(t,{frontMatter:function(){return i},contentTitle:function(){return s},metadata:function(){return p},assets:function(){return d},toc:function(){return m},default:function(){return c}});var n=a(7462),r=a(3366),l=(a(7294),a(3905)),o=["components"],i={title:"How to Create PostgreSQL Test Data",author:"Christos Hadjiaslanis",author_title:"Founder",author_url:"https://github.com/getsynth",author_image_url:"https://avatars.githubusercontent.com/u/14791384?s=460&v=4",tags:["postgres","test data","data generation","tutorial","beginners guide"],description:"This post covers three different ways to generate test data for your Postgres database",image:"https://i.imgur.com/mErPwqL.png",hide_table_of_contents:!1},s=void 0,p={permalink:"/blog/2021/03/09/postgres-data-gen",source:"@site/blog/2021-03-09-postgres-data-gen.md",title:"How to Create PostgreSQL Test Data",description:"This post covers three different ways to generate test data for your Postgres database",date:"2021-03-09T00:00:00.000Z",formattedDate:"March 9, 2021",tags:[{label:"postgres",permalink:"/blog/tags/postgres"},{label:"test data",permalink:"/blog/tags/test-data"},{label:"data generation",permalink:"/blog/tags/data-generation"},{label:"tutorial",permalink:"/blog/tags/tutorial"},{label:"beginners guide",permalink:"/blog/tags/beginners-guide"}],readingTime:6.365,truncated:!1,authors:[{name:"Christos Hadjiaslanis",title:"Founder",url:"https://github.com/getsynth",imageURL:"https://avatars.githubusercontent.com/u/14791384?s=460&v=4"}],prevItem:{title:"Why not to use prod data for testing - and what to do instead",permalink:"/blog/2021/08/04/test-data"},nextItem:{title:"Create realistic test data for your web app",permalink:"/blog/2021/03/08/mern"}},d={authorsImageUrls:[void 0]},m=[{value:"Introduction",id:"introduction",children:[]},{value:"Setup",id:"setup",children:[]},{value:"Our Schema",id:"our-schema",children:[]},{value:"Manual Insertion",id:"manual-insertion",children:[]},{value:"Using generate_series to automate the process",id:"using-generate_series-to-automate-the-process",children:[]},{value:"Using a data generator like Synth",id:"using-a-data-generator-like-synth",children:[]},{value:"Conclusion",id:"conclusion",children:[]}],u={toc:m};function c(e){var t=e.components,a=(0,r.Z)(e,o);return(0,l.kt)("wrapper",(0,n.Z)({},u,a,{components:t,mdxType:"MDXLayout"}),(0,l.kt)("h2",{id:"introduction"},"Introduction"),(0,l.kt)("p",null,"Developing high quality software inevitably requires some testing data."),(0,l.kt)("p",null,"You could be:"),(0,l.kt)("ul",null,(0,l.kt)("li",{parentName:"ul"},"Integration testing your application for correctness and regressions"),(0,l.kt)("li",{parentName:"ul"},"Testing the bounds of your application in your QA process"),(0,l.kt)("li",{parentName:"ul"},"Testing the performance of queries as the size of your dataset increases")),(0,l.kt)("p",null,"Either way, the software development lifecycle requires testing data as an integral part of developer workflow. In this article, we'll be exploring 3 different methods for generating test data for a Postgres database."),(0,l.kt)("h2",{id:"setup"},"Setup"),(0,l.kt)("p",null,"In this example we'll be using Docker to host our Postgres database."),(0,l.kt)("p",null,"To get started you'll need to ",(0,l.kt)("a",{parentName:"p",href:"https://docs.docker.com/get-docker/"},"install docker")," and start our container running Postgres:"),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-bash"},"% docker run -p 5432:5432 -d -e POSTGRES_PASSWORD=1234 -e POSTGRES_USER=postgres -e POSTGRES_DB=dev postgres\n")),(0,l.kt)("p",null,"As you can see, we've set very insecure default credentials. This is ",(0,l.kt)("em",{parentName:"p"},"not")," meant to be a robust / productionised instance, but it'll do for our testing harness."),(0,l.kt)("h2",{id:"our-schema"},"Our Schema"),(0,l.kt)("p",null,"In this example we'll setup a very simple schema. We're creating a basic app where we have a bunch of companies, and those companies have contacts."),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-sql"},"CREATE TABLE companies(\n   company_id SERIAL PRIMARY KEY,\n   company_name VARCHAR(255) NOT NULL\n);\n\nCREATE TABLE contacts(\n   contact_id SERIAL PRIMARY KEY,\n   company_id INT,\n   contact_name VARCHAR(255) NOT NULL,\n   phone VARCHAR(25),\n   email VARCHAR(100),\n   CONSTRAINT fk_company\n      FOREIGN KEY(company_id) \n      REFERENCES companies(company_id)\n);\n")),(0,l.kt)("p",null,"This schema captures some business logic of our app. We have unique primary keys, we have foreign key constraints, and we have some domain-specific data types which have 'semantic meaning'. For example, the random string ",(0,l.kt)("inlineCode",{parentName:"p"},"_SX \xc6 A-ii")," is not a valid phone number."),(0,l.kt)("p",null,"Let's get started."),(0,l.kt)("h2",{id:"manual-insertion"},"Manual Insertion"),(0,l.kt)("p",null,"The first thing you can do which works well when you're starting your project is to literally manually insert all the data you need. This involves just manually writing a SQL script with a bunch of ",(0,l.kt)("inlineCode",{parentName:"p"},"INSERT")," statements. The only thing to really think about is the insertion order so that you don't violate foreign key constraints."),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-sql"},"INSERT INTO companies(company_name)\nVALUES('BlueBird Inc'),\n      ('Dolphin LLC');     \n       \nINSERT INTO contacts(company_id, contact_name, phone, email)\nVALUES(1,'John Doe','(408)-111-1234','john.doe@bluebird.dev'),\n      (1,'Jane Doe','(408)-111-1235','jane.doe@bluebird.dev'),\n      (2,'David Wright','(408)-222-1234','david.wright@dolphin.dev');\n")),(0,l.kt)("p",null,"So here we're inserting directly into our database. This method is straight forward but does not scale when you need more data or the complexity of your schema increases. Also, testing for edge cases requires your hard-coding edge cases in the inserted data - resulting in a linear amount of work for the bugs you want to catch."),(0,l.kt)("table",null,(0,l.kt)("thead",{parentName:"table"},(0,l.kt)("tr",{parentName:"thead"},(0,l.kt)("th",{parentName:"tr",align:null},"contact_id"),(0,l.kt)("th",{parentName:"tr",align:null},"company_id"),(0,l.kt)("th",{parentName:"tr",align:null},"contact_name"),(0,l.kt)("th",{parentName:"tr",align:null},"phone"),(0,l.kt)("th",{parentName:"tr",align:null},"email"))),(0,l.kt)("tbody",{parentName:"table"},(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"John Doe"),(0,l.kt)("td",{parentName:"tr",align:null},"(408)-111-1234"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:john.doe@bluebird.dev"},"john.doe@bluebird.dev"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"2"),(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"Jane Doe"),(0,l.kt)("td",{parentName:"tr",align:null},"(408)-111-1235"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:jane.doe@bluebird.dev"},"jane.doe@bluebird.dev"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"3"),(0,l.kt)("td",{parentName:"tr",align:null},"2"),(0,l.kt)("td",{parentName:"tr",align:null},"David Wright"),(0,l.kt)("td",{parentName:"tr",align:null},"(408)-222-1234"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:david.wright@dolphin.dev"},"david.wright@dolphin.dev"))))),(0,l.kt)("h2",{id:"using-generate_series-to-automate-the-process"},"Using generate_series to automate the process"),(0,l.kt)("p",null,"Since you're a programmer, you don't like manual work. You like things to be seamless and most importantly automated!"),(0,l.kt)("p",null,"Postgres comes with a handy function called ",(0,l.kt)("inlineCode",{parentName:"p"},"generate_series")," which, ...",(0,l.kt)("em",{parentName:"p"},"drum roll"),"... generates series! We can use this to generate as much data as we want without writing it by hand."),(0,l.kt)("p",null,"Let's use ",(0,l.kt)("inlineCode",{parentName:"p"},"generate_series")," to create 100 companies and 100 contacts"),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-sql"},"INSERT INTO companies(company_name)\nSELECT md5(random()::text)\nFROM generate_series(1,100);\n\nINSERT INTO contacts(company_id, contact_name, phone, email)\nSELECT id, md5(random()::text), md5(random()::text)::varchar(20), md5(random()::text) \nFROM generate_series(1,100) id;\n")),(0,l.kt)("table",null,(0,l.kt)("thead",{parentName:"table"},(0,l.kt)("tr",{parentName:"thead"},(0,l.kt)("th",{parentName:"tr",align:null},"contact_id"),(0,l.kt)("th",{parentName:"tr",align:null},"company_id"),(0,l.kt)("th",{parentName:"tr",align:null},"contact_name"),(0,l.kt)("th",{parentName:"tr",align:null},"phone"),(0,l.kt)("th",{parentName:"tr",align:null},"email"))),(0,l.kt)("tbody",{parentName:"table"},(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"81cc02c106b7c30d4e2b032c91cdb75a"),(0,l.kt)("td",{parentName:"tr",align:null},"d056f1eee1dca55db03c"),(0,l.kt)("td",{parentName:"tr",align:null},"cd0da2eef81aaa02d6ba15ef4551fb9f")),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"2"),(0,l.kt)("td",{parentName:"tr",align:null},"2"),(0,l.kt)("td",{parentName:"tr",align:null},"d2b0112bc9bbec85c5229a4b4f28a350"),(0,l.kt)("td",{parentName:"tr",align:null},"07ba86b1dc24cdadfd24"),(0,l.kt)("td",{parentName:"tr",align:null},"7404f5b502084563f2ac20c29ed0e584")),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"3"),(0,l.kt)("td",{parentName:"tr",align:null},"3"),(0,l.kt)("td",{parentName:"tr",align:null},"64005702ecaff9f489e8074d6a718aae"),(0,l.kt)("td",{parentName:"tr",align:null},"50db9534b58e0616cd34"),(0,l.kt)("td",{parentName:"tr",align:null},"3ea36293665aa1ac38e7d6371893046a")),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"4"),(0,l.kt)("td",{parentName:"tr",align:null},"4"),(0,l.kt)("td",{parentName:"tr",align:null},"202e87bc3d0c8c080048b2c0138c709b"),(0,l.kt)("td",{parentName:"tr",align:null},"65f6ea317bd0f2c950dc"),(0,l.kt)("td",{parentName:"tr",align:null},"8b8d9b92916f4cf77c38308f6ac4391b")),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"5"),(0,l.kt)("td",{parentName:"tr",align:null},"5"),(0,l.kt)("td",{parentName:"tr",align:null},"8b2fd25d7b95158df5af671cb3255755"),(0,l.kt)("td",{parentName:"tr",align:null},"3e6ddc67aabe7164ce9a"),(0,l.kt)("td",{parentName:"tr",align:null},"ed32035400a7500203352f3597d2548f")))),(0,l.kt)("p",null,"We generated 100 companies and contacts here, the types are correct, ",(0,l.kt)("em",{parentName:"p"},"but")," the output is underwhelming. First of all, every company has exactly 1 contact, and more importantly the actual data looks completely useless. "),(0,l.kt)("p",null,"If you care about your data being semantically correct (i.e. text in your ",(0,l.kt)("inlineCode",{parentName:"p"},"phone")," column actually being a phone number) we need to get more sophisticated."),(0,l.kt)("p",null,"We could define functions ourselves to generate names / phone numbers / emails etc, but why re-invent the wheel? "),(0,l.kt)("h2",{id:"using-a-data-generator-like-synth"},"Using a data generator like Synth"),(0,l.kt)("p",null,(0,l.kt)("a",{parentName:"p",href:"https://github.com/getsynth/synth"},"Synth")," is an open-source project designed to solve the problem of creating realistic testing data. It has integration with Postgres, so you won't need to write any SQL."),(0,l.kt)("p",null,"Synth uses declarative configuration files (just JSON don't worry) to define how data should be generated. To install the ",(0,l.kt)("inlineCode",{parentName:"p"},"synth")," binary refer to the ",(0,l.kt)("a",{parentName:"p",href:"/docs/getting_started/installation"},"installation page"),"."),(0,l.kt)("p",null,"The first step to use Synth is to create a workspace. A workspace is just a directory in your filesystem that tell Synth that this is where you are going to be storing configuration:"),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-bash"},"$ mkdir workspace && cd workspace && synth init \n")),(0,l.kt)("p",null,"Next we want to create a namespace (basically a stand-alone data model) for this schema. We do this by simply creating a subdirectory and Synth will treat it as a separate schema:"),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-bash"},"$ mkdir my_app\n")),(0,l.kt)("p",null,"Now comes the fun part! Using Synth's configuration language we can specify how our data is generated. Let's start with the smaller table ",(0,l.kt)("inlineCode",{parentName:"p"},"companies"),"."),(0,l.kt)("p",null,"To tell Synth that ",(0,l.kt)("inlineCode",{parentName:"p"},"companies")," is a table (or collection in the Synth lingo) we'll create a new file ",(0,l.kt)("inlineCode",{parentName:"p"},"app/companies.json"),"."),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-json"},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "constant": 1\n    },\n    "content": {\n        "type": "object",\n        "company_id": {\n            "type": "number",\n            "id": {}\n        },\n        "company_name": {\n            "type": "string",\n            "faker": {\n                "generator": "company_name"\n            }\n        }\n    }\n}\n')),(0,l.kt)("p",null,"Here we're telling Synth that we have 2 columns, ",(0,l.kt)("inlineCode",{parentName:"p"},"company_id")," and ",(0,l.kt)("inlineCode",{parentName:"p"},"company_name"),". The first is a ",(0,l.kt)("inlineCode",{parentName:"p"},"number"),", the second is a ",(0,l.kt)("inlineCode",{parentName:"p"},"string")," and the contents of the JSON object define the constraints of the data."),(0,l.kt)("p",null,"If we sample some data using this data model we get the following:"),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-bash"},'$ synth generate my_app/ --size 2\n{\n  "companies": [\n    {\n      "company_id": 1,\n      "company_name": "Campbell Ltd"\n    },\n    {\n      "company_id": 2,\n      "company_name": "Smith PLC"\n    }\n  ]\n}\n')),(0,l.kt)("p",null,"Now we can do the same thing for the ",(0,l.kt)("inlineCode",{parentName:"p"},"contacts")," table by create a file ",(0,l.kt)("inlineCode",{parentName:"p"},"my_app/contacts.json"),". Here we have the added complexity of a foreign key constraints to the company table, but we can solve it easily using Synth's ",(0,l.kt)("a",{parentName:"p",href:"/docs/content/same-as"},(0,l.kt)("inlineCode",{parentName:"a"},"same_as"))," generator."),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-json"},'{\n    "type": "array",\n    "length": {\n        "type": "number",\n        "constant": 1\n    },\n    "content": {\n        "type": "object",\n        "company_id": {\n            "type": "same_as",\n            "ref":"companies.content.company_id"\n        },\n        "contact_name": {\n            "type": "string",\n            "faker": {\n                "generator": "name"\n            }\n        },\n        "phone": {\n            "type": "string",\n            "faker": {\n                "generator": "phone_number",\n                "locales": ["en_GB"]\n            }\n        },\n        "email": {\n            "type": "string",\n            "faker": {\n                "generator": "safe_email"\n            }\n        }\n    }\n}\n')),(0,l.kt)("p",null,"There is quite a bit going on here - to get an in-depth understanding of the synth configuration refer I'd recommend reading the comprehensive docs. There are tons of cool features which this schema can't really explore!"),(0,l.kt)("p",null,"Now we have both our tables data model under Synth, we can generate data into Postgres:"),(0,l.kt)("pre",null,(0,l.kt)("code",{parentName:"pre",className:"language-bash"},"$ synth generate my_app/ --to postgres://postgres:1234@localhost:5432/dev\n")),(0,l.kt)("p",null,"Taking a look at the company table:"),(0,l.kt)("table",null,(0,l.kt)("thead",{parentName:"table"},(0,l.kt)("tr",{parentName:"thead"},(0,l.kt)("th",{parentName:"tr",align:null},"contact_id"),(0,l.kt)("th",{parentName:"tr",align:null},"company_id"),(0,l.kt)("th",{parentName:"tr",align:null},"contact_name"),(0,l.kt)("th",{parentName:"tr",align:null},"phone"),(0,l.kt)("th",{parentName:"tr",align:null},"email"))),(0,l.kt)("tbody",{parentName:"table"},(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"1"),(0,l.kt)("td",{parentName:"tr",align:null},"Carrie Walsh"),(0,l.kt)("td",{parentName:"tr",align:null},"+44(0)117 496 0785"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:espinozabetty@hotmail.com"},"espinozabetty@hotmail.com"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"2"),(0,l.kt)("td",{parentName:"tr",align:null},"2"),(0,l.kt)("td",{parentName:"tr",align:null},"Brittany Flores"),(0,l.kt)("td",{parentName:"tr",align:null},"+441632 960 480"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:osharp@mcdaniel.com"},"osharp@mcdaniel.com"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"3"),(0,l.kt)("td",{parentName:"tr",align:null},"3"),(0,l.kt)("td",{parentName:"tr",align:null},"Tammy Rodriguez"),(0,l.kt)("td",{parentName:"tr",align:null},"01632960737"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:brenda82@ward.org"},"brenda82@ward.org"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"4"),(0,l.kt)("td",{parentName:"tr",align:null},"4"),(0,l.kt)("td",{parentName:"tr",align:null},"Amanda Marks"),(0,l.kt)("td",{parentName:"tr",align:null},"(0808) 1570096"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:hwilcox@gonzalez.com"},"hwilcox@gonzalez.com"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"5"),(0,l.kt)("td",{parentName:"tr",align:null},"5"),(0,l.kt)("td",{parentName:"tr",align:null},"Kimberly Delacruz MD"),(0,l.kt)("td",{parentName:"tr",align:null},"+44(0)114 4960207"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:pgarcia@thompson.com"},"pgarcia@thompson.com"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"6"),(0,l.kt)("td",{parentName:"tr",align:null},"6"),(0,l.kt)("td",{parentName:"tr",align:null},"Jordan Williamson"),(0,l.kt)("td",{parentName:"tr",align:null},"(0121) 4960483"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:jamesmiles@weber.org"},"jamesmiles@weber.org"))),(0,l.kt)("tr",{parentName:"tbody"},(0,l.kt)("td",{parentName:"tr",align:null},"7"),(0,l.kt)("td",{parentName:"tr",align:null},"7"),(0,l.kt)("td",{parentName:"tr",align:null},"Nicholas Williams"),(0,l.kt)("td",{parentName:"tr",align:null},"(0131) 496 0974"),(0,l.kt)("td",{parentName:"tr",align:null},(0,l.kt)("a",{parentName:"td",href:"mailto:fordthomas@gmail.com"},"fordthomas@gmail.com"))))),(0,l.kt)("p",null,"Much better :)"),(0,l.kt)("h2",{id:"conclusion"},"Conclusion"),(0,l.kt)("p",null,"We explored 3 different ways to generate data."),(0,l.kt)("ul",null,(0,l.kt)("li",{parentName:"ul"},(0,l.kt)("strong",{parentName:"li"},"Manual Insertion"),": Is ok to get you started. If your needs are basic it's the path of least effort to creating a working dataset."),(0,l.kt)("li",{parentName:"ul"},(0,l.kt)("strong",{parentName:"li"},"Postgres generate_series"),": This method scales better than manual insertion - but if you care about the contents of your data and have foreign key constraints you'll need to write quite a bit of bespoke SQL by hand."),(0,l.kt)("li",{parentName:"ul"},(0,l.kt)("a",{parentName:"li",href:"https://github.com/getsynth/synth"},(0,l.kt)("strong",{parentName:"a"},"Synth")),": Synth has a small learning curve, but to create realistic testing data at scale it reduces most of the manual labour.")),(0,l.kt)("p",null,"In the next post we'll explore how to subset your existing database for testing purposes. And don't worry if you have sensitive / personal data - we'll cover that too."))}c.isMDXComponent=!0}}]);