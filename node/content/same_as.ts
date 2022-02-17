import {IContent} from './content';

type SameAsContent = string | QualifiedSameAs;

interface QualifiedSameAs extends IContent {
    type: 'same_as',
    ref: string
}

/**
 * Create a new 'same as' generator.
 * @param ref The string reference to some node in the schema tree.
 * @return {Content} A 'same as' content/schema node.
 */
function sameAs(ref: string): QualifiedSameAs {
  return {
    type: 'same_as',
    ref,
  };
}

export {SameAsContent, sameAs};
