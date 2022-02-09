import { IContent, Content } from "./content"

interface ObjContent extends IContent {
    type: "object"
}

function obj(content: Record<string, Content>): ObjContent {
    return {
        type: "object",
        ...content
    }
}

export { obj, ObjContent }
