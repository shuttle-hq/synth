"use strict";

const neon = require("./index.node");

class Content {
    #content

    /**
     * Construct a new Synth schema which can then be sampled.
     * @param schema The Synth schema to construct this object from.
     */
    constructor(schema) {
        this.#content = neon.new_content(schema);
    }

    /**
     * Create a new sampler that will generated data based on this schema.
     * @param seed The RNG seed to use when sampling.
     */
    sample(seed = 0) {
        return new Sampler(neon.new_sampler(this.#content, seed));
    }

    /**
     * Create a new sampler with a random RNG seed.
     */
    sampleRandomSeed() {
        return new Sampler(neon.new_sampler_random_seed(this.#content));
    }
}

class Sampler {
    #sampler

    constructor(internal) {
        this.#sampler = internal;
    }

    /*
     * Sample a value from the schema.
     */
    next() {
        return neon.sampler_next(this.#sampler);
    }

    [Symbol.iterator]() {
        return {
            next: () => ({ value: this.next(), done: false })
        }
    }
}

module.exports = require("./tsbuild");
module.exports.Content = Content;
