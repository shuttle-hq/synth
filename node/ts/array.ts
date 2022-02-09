import { Content, IContent } from "./content"

interface ArrayContent extends IContent {
    type: "array",
    length: Content,
    content: Content
}

function array(length: Content, content: Content): ArrayContent {
    return {
        type: "array",
        length,
        content
    }
}

export { array, ArrayContent }
