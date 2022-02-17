import {Content, IContent} from './content';

interface ArrayContent extends IContent {
    type: 'array',
    length: Content,
    content: Content
}
/**
 * Create a new array schema node.
 * @param length Length of the array.
 * @param content Content contained within the array.
 * @return {Content} An array content/schema node.
 */
function array(length: Content, content: Content): ArrayContent {
  return {
    type: 'array',
    length,
    content,
  };
}

export {array, ArrayContent};
