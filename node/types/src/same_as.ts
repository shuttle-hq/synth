import { IContent } from "./content"

type SameAsContent = string | QualifiedSameAs;

interface QualifiedSameAs extends IContent {
    ref: string
}

const SameAs = function (ref: string): QualifiedSameAs {
    return {
        type: "same_as",
        ref
    }
}

export { SameAs, SameAsContent, QualifiedSameAs }
export default SameAs
