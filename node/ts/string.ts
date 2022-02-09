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
    | "free_email_provider"
    | "domain_suffix"
    | "free_email"
    | "safe_email"
    | "username"
    | "ipv4"
    | "ipv6"
    | "ip"
    | "mac_address"
    | "color"
    | "user_agent"
    | "rfc_status_code"
    | "valid_status_code"
    | "company_suffix"
    | "company_name"
    | "buzzword"
    | "buzzword_muddle"
    | "buzzword_tail"
    | "catch_phrase"
    | "bs_verb"
    | "bs_adj"
    | "bs_noun"
    | "bs"
    | "profession"
    | "industry"
    | "city_prefix"
    | "city_suffix"
    | "city_name"
    | "country_name"
    | "country_code"
    | "street_suffix"
    | "street_name"
    | "time_zone"
    | "state_name"
    | "state_abbr"
    | "secondary_address_type"
    | "secondary_address"
    | "zip_code"
    | "post_code"
    | "building_number"
    | "latitude"
    | "longitude"
    | "phone_number"
    | "cell_number"
    | "file_path"
    | "file_name"
    | "file_extension"
    | "dir_path"
    | "ascii_email"
    | "ascii_free_email"
    | "ascii_safe_email"
    | "address"

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
