import {StringContent, str} from './string';
import {NumberContent, number} from './number';
import {BoolContent, constantBool, randomBool} from './bool';
import {ObjContent, obj} from './object';
import {ArrayContent, array} from './array';
import {OneOfContent, oneOf} from './one_of';
import {SameAsContent, sameAs} from './same_as';
import {DateTimeContent, dateTime} from './date_time';

type Content =
    StringContent
    | NumberContent
    | BoolContent
    | ObjContent
    | ArrayContent
    | OneOfContent
    | SameAsContent
    | DateTimeContent;

type ContentType =
    'string'
    | 'number'
    | 'object'
    | 'array'
    | 'bool'
    | 'one_of'
    | 'same_as'
    | 'date_time';

interface IContent {
    type: ContentType
}

export {
  Content, IContent, str, number, constantBool, randomBool, obj, array, oneOf,
  sameAs, dateTime,
};
export default Content;
