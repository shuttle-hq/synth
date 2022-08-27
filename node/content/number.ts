import {IContent} from './content';

type NumberSubtype = 'u64' | 'i64' | 'f64' | 'u32' | 'i32';

type NumberContent = Constant | Range | Id;

interface Number extends IContent {
    type: 'number',
    subtype: NumberSubtype
}

interface Id extends Number {
    subtype: 'u64',
    id: {
        start_at?: number
    }
}

/**
 * Identifies the most appropriate subtype for a number generator based on the
 * constant value or range values provided.
 * @param values The range or constant values from which to determine the best
 * subtype for a particular number generator.
 * @return {NumberSubtype} The most suitable number subtype.
 */
function bestSubtype(...values: number[]): NumberSubtype {
  if (values.length < 2) {
    const n = values[0];
    if (Number.isInteger(n)) {
      return n >= 0 ? 'u64' : 'i64';
    } else {
      return 'f64';
    }
  } else {
    return values.map((value) => bestSubtype(value)).sort().slice(-1)[0];
  }
}

interface Range extends Number {
    range: {
        low: number,
        high: number,
        step: number
    }
}

type Constant = QualifiedConstant | LiteralConstant;

type LiteralConstant = number;

interface QualifiedConstant extends Number {
    constant: number
}

const number = {
  id: function(start_at: number = 0): Id {
    return {
      type: 'number',
      subtype: 'u64',
      id: {
        start_at,
      },
    };
  },
  constant: function(constant: number): QualifiedConstant {
    return {
      type: 'number',
      subtype: bestSubtype(constant),
      constant,
    };
  },

  range: function(low: number, high: number, step: number = 1): Range {
    return {
      type: 'number',
      subtype: bestSubtype(low, high, step),
      range: {
        low,
        high,
        step,
      },
    };
  },
};


export {number, NumberContent};
