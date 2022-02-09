"use strict";

const neon = require("./index.node");

class Content {
    constructor(schema) { this._content = neon.new_content(schema); }
    sample() { return new Sampler(this._content); }
}

class Sampler {
    constructor(content, seed = 0) { this._sampler = neon.new_sampler(content); }
    next() { neon.sampler_next(this._sampler); }
    [Symbol.iterator]() { return this; }
}

module.exports.Content = Content;
module.exports.Sampler = Sampler;
