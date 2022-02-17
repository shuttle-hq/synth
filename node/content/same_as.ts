import { IContent } from "./content"

type SameAsContent = string | QualifiedSameAs;

interface QualifiedSameAs extends IContent {
    type: "same_as",
    ref: string
}

function sameAs(ref: string): QualifiedSameAs {
    return {
        type: "same_as",
        ref
    }
}

export { SameAsContent, sameAs }
