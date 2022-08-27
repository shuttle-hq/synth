import {Content, IContent} from './content';

interface OneOfContent extends IContent {
    type: 'one_of',
    variants: Content[]
}

/**
 * Create a new 'one of' generator.
 * @param variants Set of generators that may be choosen when sampling.
 * @return {Content} A 'one of' content/schema node.
 */
function oneOf(...variants: Content[]): OneOfContent {
  return {
    type: 'one_of',
    variants,
  };
}

export {OneOfContent, oneOf};
