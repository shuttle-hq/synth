const neon: any = require('../index.node');
import {
  Content,
  str, number, constantBool, randomBool, obj, array, oneOf, sameAs,
} from './content/content';

class Compiled {
  private _content: any;

  /**
     * Construct a new Synth schema which can then be sampled.
     * @param schema The Synth schema to construct this object from.
     */
  constructor(schema: Content) {
    this._content = neon.new_content(schema);
  }

  /**
     * Create a new sampler that will generated data based on this schema.
     * @param seed The RNG seed to use when sampling.
     * @return {Sampler} A new sampler based on this compiled schema.
     */
  sample(seed: number = 0) {
    return new Sampler(neon.new_sampler(this._content, seed));
  }

  /**
     * Create a new sampler with a random RNG seed.
     * @return {Sampler} A new randomly-seed sampler based on this compiled
     * schema.
     */
  sampleRandomSeed() {
    return new Sampler(neon.new_sampler_random_seed(this._content));
  }
}

class Sampler {
  private _sampler: any;

  constructor(internal: any) {
    this._sampler = internal;
  }

  /*
     * Sample a value from the schema.
     */
  next() {
    return neon.sampler_next(this._sampler);
  }

  [Symbol.iterator]() {
    return {
      next: () => ({value: this.next(), done: false}),
    };
  }
}

export {
  Compiled, Content,
  str, number, constantBool, randomBool, obj, array, oneOf, sameAs,
};
