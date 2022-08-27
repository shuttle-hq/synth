import {IContent, Content} from './content';

interface ObjContent extends IContent {
    type: 'object'
}

/**
 * Create a number object generator.
 * @param content The JavaScript object/record mapping object keys to schema
 * nodes.
 * @return {Content} An object schema/content node.
 */
function obj(content: Record<string, Content>): ObjContent {
  return {
    type: 'object',
    ...content,
  };
}

export {obj, ObjContent};
