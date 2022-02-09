import { IContent, Content } from "./content"

interface ObjContent extends IContent {
    type: "object"
}

const Obj = function (content: Record<string, Content>): ObjContent {
    return {
        type: "object",
        ...content
    }
}

export { Obj, ObjContent }
