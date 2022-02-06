import { StringContent } from "./string"
import { NumberContent } from "./number"
import { BoolContent } from "./bool"
import { ObjectContent } from "./object"
import { ArrayContent } from "./array"
import { OneOfContent } from "./one_of"
import { SameAsContent } from "./same_as"

type Content =
    StringContent
    | NumberContent
    | BoolContent
    | ObjectContent
    | ArrayContent
    | OneOfContent
    | SameAsContent;

type ContentType =
    "string"
    | "date_time"
    | "number"
    | "object"
    | "array"
    | "bool"
    | "one_of"
    | "same_as";

interface IContent {
    type: ContentType
}

export { Content, IContent }
export default Content
