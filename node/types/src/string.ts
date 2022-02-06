import { Content, IContent } from "./content"

type StringContent = Format | Pattern | Faker | Categorical;

interface Pattern extends IContent {
    pattern: string // TODO: regex
}

const Pattern = function (pattern: string): Pattern {
    return {
        type: "string",
        pattern
    }
}

interface Format extends IContent {
    format: {
        format: string,
        arguments: Record<string, Content>
    }
}

const Format = function (format: string, args: Record<string, Content>): Format {
    return {
        type: "string",
        format: {
            format,
            arguments: args
        }
    }
}

type FakerGenerator =
    "first_name"
    | "last_name"
    | "title"
    | "suffix"
    | "name"
    | "name_with_title"
    | "credit_card"
    | "free_email_provider" // TODO: See graph/string/faker.rs

interface Faker extends IContent {
    faker: {
        generator: FakerGenerator
    }
}

const Faker = function (generator: FakerGenerator): Faker {
    return {
        type: "string",
        faker: {
            generator
        }
    }
}

interface Categorical extends IContent {
    categorical: Record<string, number>
}

const Categorical = function (categorical: Record<string, number>): Categorical {
    return {
        type: "string",
        categorical
    }
}

const String = {
    format: Format,
    pattern: Pattern,
    faker: Faker,
    categorical: Categorical
}

export { Pattern, Format, String, Faker, Categorical, StringContent }
export default String;
