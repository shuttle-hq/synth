import { IContent } from "./content"

type NumberSubtype = "u64" | "i64" | "f64" | "u32" | "i32";

type NumberContent = Constant | Range | Id;

interface Number extends IContent {
    type: "number",
    subtype: NumberSubtype
}

interface Id extends Number {
    subtype: "u64",
    id: {
        start_at?: number
    }
}

function idNumber(start_at?: number): Id {
    return {
        type: "number",
        subtype: "u64",
        id: {
            start_at
        }
    }
}

function bestSubtype(...values: number[]): NumberSubtype {
    if (values.length < 2) {
        const n = values[0];
        if (Number.isInteger(n)) {
            return n >= 0 ? "u64" : "i64"
        } else {
            return "f64"
        }
    } else {
        return values.map((value) => bestSubtype(value)).sort().slice(-1)[0]
    }
}

interface Range extends Number {
    range: {
        low: number,
        high: number,
        step: number
    }
}

function rangeNumber(low: number, high: number, step: number = 1): Range {
    return {
        type: "number",
        subtype: bestSubtype(low, high, step),
        range: {
            low,
            high,
            step
        }
    }
}

type Constant = QualifiedConstant | LiteralConstant;

type LiteralConstant = number;

interface QualifiedConstant extends Number {
    constant: number
}

function constantNumber(constant: number): QualifiedConstant {
    return {
        type: "number",
        subtype: bestSubtype(constant),
        constant
    }
}

export { idNumber, rangeNumber, constantNumber, NumberContent }
