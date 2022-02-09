"use strict";

const neon = require("./index.node");

class Content {
    /**
     * Construct a new Synth schema which can then be sampled.
     * @param schema The Synth schema to construct this object from.
     */
    constructor(schema) {
        this._content = neon.new_content(schema);
    }

    /**
     * Create a new sampler that will generated data based on this schema.
     * @param seed The RNG seed to use when sampling.
     */
    sample(seed = 0) {
        return new Sampler(this._content, seed);
    }
}

class Sampler {
    /**
     * Construct a Synth sampler to generate values based on some schema.
     * @param content The compiled schema from which to sample.
     * @param seed The RNG seed to use when sampling.
     */
    constructor(content, seed = 0) {
        this._sampler = neon.new_sampler(content, seed);
    }

    /*
     * Sample a value from the schema.
     */
    next() {
        return neon.sampler_next(this._sampler);
    }

    [Symbol.iterator]() {
        return {
            next: () => ({ value: this.next(), done: false })
        }
    }
}

module.exports.schema = require("./tsbuild");
module.exports.Content = Content;
module.exports.Sampler = Sampler;
