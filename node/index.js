"use strict";

const neon = require("./index.node");

class Content {
    constructor(schema) {
        this._content = neon.new_content(schema);
    }

    sample(seed = 0) {
        return new Sampler(this._content, seed);
    }
}

class Sampler {
    constructor(content, seed = 0) {
        this._sampler = neon.new_sampler(content, seed);
    }

    next() {
        return neon.sampler_next(this._sampler);
    }

    [Symbol.iterator]() {
        return {
            next: () => ({ value: this.next(), done: false })
        }
    }
}

module.exports.Content = Content;
module.exports.Sampler = Sampler;
