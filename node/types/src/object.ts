import { IContent, Content } from "./content"

interface ObjectContent extends IContent {
    type: "object"
}

const Object = function (content: Record<string, Content>): ObjectContent {
    return {
        type: "object",
        ...content
    }
}

export { Object, ObjectContent }
