import { IContent } from "./content";

interface DateTime extends IContent {
    format: string,
    begin: string,
    end: string
}

const DateTime = function (begin: Date, end: Date, format: string): DateTime {
    const strftime = require("strftime");
    return {
        type: "date_time",
        format,
        begin: strftime(format, begin),
        end: strftime(format, end)
    }
}

export { DateTime }
