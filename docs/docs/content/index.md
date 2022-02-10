---
title: All generators
---

Synth has the following types of generators:

## Generators

- [null](null) generates as many nulls as you ever want
- [bool](bool) generates `true` or `false`, either constant or
  following a given percentage
- [number](number) generates ranges, distributions or series of
  [integer or floating-point](number#subtype) numbers
- [series](series) generates streams of events (e.g. for logs)
  - [incrementing](series#incrementing) emits evenly spaced events
  - [poisson](series#poisson) models a random poisson process
  - [cyclical](series#cyclical) models periodic events
  - [zip](series#zip) combines multiple series together
- [string](string) can contain one of the following generators for
  various classes of string:
  - [pattern](string#pattern) takes a regular expression and
    generates matching strings
  - [uuid](string#uuid) generates hyphenated UUIDs
    optionally with time zone
  - [faker](string#faker) has a large number of generators for names,
    contact information, credit card numbers, sentences, and much more
  - [format](string#format) combines multiple generators to one
    formatted string
  - [serialized](string#serialized) JSONifies the value of the
    contained generator
  - [truncated](string#truncated) ensures all generated strings stay
    within length limits
  - [categorical](string#categorical) is like a
    [one_of](one-of) specialized for strings
- [date_time](date-time) generates dates and times
- [object](object) creates an object with string keys containing
  generators for the values
- [array](array) fills an array of the given length with elements of
  the contained generator
- [datasource](datasource) pulls data from an external source
  like a file

## Modifiers

[Modifiers](modifiers) encode additional constraints or variations of the generator(s) they apply to.

- [optional](modifiers#optional) makes a value nullable
- [unique](modifiers#unique) ensures the generated values don't contain
  duplicates
- [one_of](one-of) allows you to choose from a set of contained
  generators
- [same_as](same-as) creates a reference to another field in this or
  another collection
