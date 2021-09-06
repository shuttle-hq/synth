(window.webpackJsonp=window.webpackJsonp||[]).push([[14],{128:function(e,n,t){"use strict";t.d(n,"a",(function(){return s})),t.d(n,"b",(function(){return d}));var r=t(0),a=t.n(r);function i(e,n,t){return n in e?Object.defineProperty(e,n,{value:t,enumerable:!0,configurable:!0,writable:!0}):e[n]=t,e}function o(e,n){var t=Object.keys(e);if(Object.getOwnPropertySymbols){var r=Object.getOwnPropertySymbols(e);n&&(r=r.filter((function(n){return Object.getOwnPropertyDescriptor(e,n).enumerable}))),t.push.apply(t,r)}return t}function c(e){for(var n=1;n<arguments.length;n++){var t=null!=arguments[n]?arguments[n]:{};n%2?o(Object(t),!0).forEach((function(n){i(e,n,t[n])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(t)):o(Object(t)).forEach((function(n){Object.defineProperty(e,n,Object.getOwnPropertyDescriptor(t,n))}))}return e}function p(e,n){if(null==e)return{};var t,r,a=function(e,n){if(null==e)return{};var t,r,a={},i=Object.keys(e);for(r=0;r<i.length;r++)t=i[r],n.indexOf(t)>=0||(a[t]=e[t]);return a}(e,n);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(r=0;r<i.length;r++)t=i[r],n.indexOf(t)>=0||Object.prototype.propertyIsEnumerable.call(e,t)&&(a[t]=e[t])}return a}var l=a.a.createContext({}),b=function(e){var n=a.a.useContext(l),t=n;return e&&(t="function"==typeof e?e(n):c(c({},n),e)),t},s=function(e){var n=b(e.components);return a.a.createElement(l.Provider,{value:n},e.children)},u={inlineCode:"code",wrapper:function(e){var n=e.children;return a.a.createElement(a.a.Fragment,{},n)}},m=a.a.forwardRef((function(e,n){var t=e.components,r=e.mdxType,i=e.originalType,o=e.parentName,l=p(e,["components","mdxType","originalType","parentName"]),s=b(t),m=r,d=s["".concat(o,".").concat(m)]||s[m]||u[m]||i;return t?a.a.createElement(d,c(c({ref:n},l),{},{components:t})):a.a.createElement(d,c({ref:n},l))}));function d(e,n){var t=arguments,r=n&&n.mdxType;if("string"==typeof e||r){var i=t.length,o=new Array(i);o[0]=m;var c={};for(var p in n)hasOwnProperty.call(n,p)&&(c[p]=n[p]);c.originalType=e,c.mdxType="string"==typeof e?e:r,o[1]=c;for(var l=2;l<i;l++)o[l]=t[l];return a.a.createElement.apply(null,o)}return a.a.createElement.apply(null,t)}m.displayName="MDXCreateElement"},82:function(e,n,t){"use strict";t.r(n),t.d(n,"frontMatter",(function(){return c})),t.d(n,"metadata",(function(){return p})),t.d(n,"toc",(function(){return l})),t.d(n,"default",(function(){return s}));var r=t(3),a=t(7),i=(t(0),t(128)),o=["components"],c={},p={unversionedId:"content/number",id:"content/number",isDocsHomePage:!1,title:"number",description:"Synth's number type allows for generating fixed-width numbers.",source:"@site/docs/content/number.md",slug:"/content/number",permalink:"/synth/content/number",editUrl:"https://github.com/getsynth/synth/edit/master/docs/docs/content/number.md",version:"current",sidebar:"docsSidebar",previous:{title:"bool",permalink:"/synth/content/bool"},next:{title:"string",permalink:"/synth/content/string"}},l=[{value:"Parameters",id:"parameters",children:[]},{value:"range",id:"range",children:[]},{value:"constant",id:"constant",children:[]},{value:"id",id:"id",children:[]}],b={toc:l};function s(e){var n=e.components,t=Object(a.a)(e,o);return Object(i.b)("wrapper",Object(r.a)({},b,t,{components:n,mdxType:"MDXLayout"}),Object(i.b)("p",null,"Synth's ",Object(i.b)("inlineCode",{parentName:"p"},"number")," type allows for generating fixed-width numbers. "),Object(i.b)("h3",{id:"parameters"},"Parameters"),Object(i.b)("h4",{id:"subtype"},Object(i.b)("inlineCode",{parentName:"h4"},"subtype")),Object(i.b)("p",null,"All the variants of ",Object(i.b)("inlineCode",{parentName:"p"},"number")," accept an optional ",Object(i.b)("inlineCode",{parentName:"p"},'"subtype"')," field to specify\nthe width and primitive kind of the values generated. The value of ",Object(i.b)("inlineCode",{parentName:"p"},'"subtype"'),",\nif specified, must be one of ",Object(i.b)("inlineCode",{parentName:"p"},"u64"),", ",Object(i.b)("inlineCode",{parentName:"p"},"i64"),", ",Object(i.b)("inlineCode",{parentName:"p"},"f64"),", ",Object(i.b)("inlineCode",{parentName:"p"},"u32"),", ",Object(i.b)("inlineCode",{parentName:"p"},"i32"),", ",Object(i.b)("inlineCode",{parentName:"p"},"f32"),"."),Object(i.b)("h4",{id:"example"},"Example"),Object(i.b)("pre",null,Object(i.b)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "number",\n  "subtype": "u32",\n  "constant": 42\n}\n')),Object(i.b)("p",null,"It is not required to specify the ",Object(i.b)("inlineCode",{parentName:"p"},'"subtype"')," field: ",Object(i.b)("inlineCode",{parentName:"p"},"synth")," will try to infer\nthe best value based on the value of other parameters. But it may be necessary\nto set it manually in situations where the data sink only accepts certain\nwidths (e.g. postgres)."),Object(i.b)("h2",{id:"range"},"range"),Object(i.b)("p",null,"Defines a range over a semi-open interval ",Object(i.b)("inlineCode",{parentName:"p"},"[low, high)")," with a specified ",Object(i.b)("inlineCode",{parentName:"p"},"step"),". Values are generated by sampling a\nrandom non-negative integer ",Object(i.b)("inlineCode",{parentName:"p"},"n")," and computing ",Object(i.b)("inlineCode",{parentName:"p"},"low + n*step"),"."),Object(i.b)("h4",{id:"example-1"},"Example"),Object(i.b)("pre",null,Object(i.b)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "number",\n  "range": {\n      "high": 15000000.0, // temperature at sun\'s core in Celcius\n      "low": -273.15,     // 0 Kelvin\n      "step": 0.01\n  }\n}\n')),Object(i.b)("h2",{id:"constant"},"constant"),Object(i.b)("p",null,"A constant number type. This will always evaluate to the same number."),Object(i.b)("h4",{id:"example-2"},"Example"),Object(i.b)("pre",null,Object(i.b)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "number",\n  "constant": 3.14159  // pi\n}\n')),Object(i.b)("p",null,"The constant number generator can also be simply declared by its desired output value."),Object(i.b)("h4",{id:"example-3"},"Example"),Object(i.b)("p",null,"The schema"),Object(i.b)("pre",null,Object(i.b)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "object",\n  "just_the_number_42": 42\n}\n')),Object(i.b)("p",null,"is the same as the longer"),Object(i.b)("pre",null,Object(i.b)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "object",\n  "just_the_number_42": {\n    "type": "number",\n    "constant": 42\n  }\n}\n')),Object(i.b)("h2",{id:"id"},"id"),Object(i.b)("p",null,"A monotonically increasing number type, most commonly used as a unique row identifier. The optional ",Object(i.b)("inlineCode",{parentName:"p"},"start")," field\ndefaults to 1 if unspecified."),Object(i.b)("p",null,"Synth currently supports ",Object(i.b)("inlineCode",{parentName:"p"},"u64")," ids."),Object(i.b)("h4",{id:"example-4"},"Example"),Object(i.b)("pre",null,Object(i.b)("code",{parentName:"pre",className:"language-json",metastring:"synth",synth:!0},'{\n  "type": "array",\n  "length": {\n    "type": "number",\n    "constant": 5\n  },\n  "content": {\n    "type": "number",\n    "id": {\n      "start_at": 10\n    }\n  }\n}\n')))}s.isMDXComponent=!0}}]);