import { Content, IContent } from "./content"

interface OneOfContent extends IContent {
    type: "one_of",
    variants: Content[]
}

function oneOf(...variants: Content[]): OneOfContent {
    return {
        type: "one_of",
        variants
    }
}

export { OneOfContent, oneOf }
