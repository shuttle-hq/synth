import { Content, IContent } from "./content"

interface OneOfContent extends IContent {
    type: "one_of",
    variants: Content[]
}

const OneOf = function (...variants: Content[]): OneOfContent {
    return {
        type: "one_of",
        variants
    }
}

export { OneOfContent, OneOf }
export default OneOf
