import { IContent } from "./content";

type BoolContent = ConstantBool | RandomBool;

type ConstantBool = boolean | QualifiedBool;

interface Bool extends IContent {
    type: "bool"
}

interface QualifiedBool extends Bool {
    constant: boolean
}

function constantBool(constant: boolean): QualifiedBool {
    return {
        type: "bool",
        constant
    }
}

interface RandomBool extends Bool {
    frequency: number
}

function randomBool(frequency: number): RandomBool {
    return {
        type: "bool",
        frequency
    }
}

export { BoolContent, constantBool, randomBool }
