import { IContent } from "./content";

type BoolContent = ConstantBool | RandomBool;

type ConstantBool = boolean | QualifiedBool;

interface QualifiedBool extends IContent {
    constant: boolean
}

const ConstantBool = function (constant: boolean): QualifiedBool {
    return {
        type: "bool",
        constant
    }
}

interface RandomBool extends IContent {
    frequency: number
}

const RandomBool = function (frequency: number): RandomBool {
    return {
        type: "bool",
        frequency
    }
}

const Bool = {
    random: RandomBool,
    constant: ConstantBool
}

export { BoolContent, Bool, ConstantBool, RandomBool }
export default Bool
