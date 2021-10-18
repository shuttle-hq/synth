---
title: Table of Contents
---

Synth has the following types of generators:

## [Modifiers](/modifiers)

Modifiers encode additional constraints or variations on their contained
generator(s)

* [optional](/modifiers#optional) makes a value nullable
* [unique](/modifiers#unique) ensures the generated values don't contain
duplicates
* [one_of](/content/one-of) allows you to choose from a set of contained
generators
* [same_as](/content/same-as) creates a reference to another field in this or
another collection

## Generators

* [null](/content/null) generates as many nulls as you ever want
* [bool](/content/bool) generates `true` or `false`, either constant or
following a given percentage
* [number](/content/number) generates ranges, distributions or series of
[integer or floating-point](/content/number#subtype) numbers
* [series](/content/series) generates streams of events (e.g. for logs)
  * [incrementing](/content/series#incrementing) emits evenly spaced events
  * [poisson](/content/series#poisson) models a random poisson process
  * [cyclical](/content/series#cyclical) models periodic events
  * [zip](/content/series#zip) combines multiple series together
* [string](/content/string) can contain one of the following generators for
various classes of string:
  * [pattern](/content/string#pattern) takes a regular expression and
  generates matching strings
  * [uuid](/content/string#uuid) generates hyphenated UUIDs
  optionally with time zone
  * [faker](/content/string#faker) has a large number of generators for names,
  contact information, credit card numbers, sentences, and much more
  * [format](/content/string#format) combines multiple generators to one
  formatted string
  * [serialized](/content/string#serialized) JSONifies the value of the
  contained generator
  * [truncated](/content/string#truncated) ensures all generated strings stay
  within length limits
  * [categorical](/content/string#categorical) is like a
  [one_of](/content/one-of) specialized for strings
* [date_time](/content/date-time) generates dates and times,
* [object](/content/object) creates an object with string keys containing
generators for the values
* [array](/content/array) fills an array of the given length with elements of
the contained generator
