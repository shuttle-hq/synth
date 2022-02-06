import { Content, IContent } from "./content"

interface ArrayContent extends IContent {
    length: Content,
    content: Content
}

const Array = function (length: Content, content: Content): ArrayContent {
    return {
        type: "array",
        length,
        content
    }
}

export { Array, ArrayContent }
export default Array
