import {IContent} from './content';

type BoolContent = ConstantBool | RandomBool;

type ConstantBool = boolean | QualifiedBool;

interface Bool extends IContent {
    type: 'bool'
}

interface QualifiedBool extends Bool {
    constant: boolean
}

/**
 * Create a new constant Boolean value generator.
 * @param constant The constant Boolean value to return.
 * @return {Content} Boolean content/schema node.
 */
function constantBool(constant: boolean): QualifiedBool {
  return {
    type: 'bool',
    constant,
  };
}

interface RandomBool extends Bool {
    frequency: number
}

/**
 * Create a random Boolean value generator.
 * @param frequency Float between `0.0` and `1.0` that indicates the probability
 * of a `true` value being returned.
 * @return {Content} Boolean content/schema node.
 */
function randomBool(frequency: number): RandomBool {
  return {
    type: 'bool',
    frequency,
  };
}

export {BoolContent, constantBool, randomBool};
