import {IContent} from './content';
const strftime = require('strftime');

interface DateTimeContent extends IContent {
    format: string,
    begin: string,
    end: string
}

/**
 * Create a new date/time generator.
 * @param begin The start of the date/time range.
 * @param end The end of the date/time range.
 * @param format The 'strftime' format string.
 * @return {Content} A The new date/time generator schema node.
 */
function dateTime(begin: Date, end: Date, format: string): DateTimeContent {
  return {
    type: 'date_time',
    format,
    begin: strftime(format, begin),
    end: strftime(format, end),
  };
};

export {DateTimeContent, dateTime};
