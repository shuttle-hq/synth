import {Content, IContent} from './content';

type StringContent = Format | Pattern | Faker | Categorical;

interface String extends IContent {
    type: 'string'
}

interface Pattern extends String {
    pattern: string
}

interface Format extends String {
    format: {
        format: string,
        arguments: Record<string, Content>
    }
}

type FakerGenerator =
    'first_name'
    | 'last_name'
    | 'title'
    | 'suffix'
    | 'name'
    | 'name_with_title'
    | 'credit_card'
    | 'free_email_provider'
    | 'domain_suffix'
    | 'free_email'
    | 'safe_email'
    | 'username'
    | 'ipv4'
    | 'ipv6'
    | 'ip'
    | 'mac_address'
    | 'color'
    | 'user_agent'
    | 'rfc_status_code'
    | 'valid_status_code'
    | 'company_suffix'
    | 'company_name'
    | 'buzzword'
    | 'buzzword_muddle'
    | 'buzzword_tail'
    | 'catch_phrase'
    | 'bs_verb'
    | 'bs_adj'
    | 'bs_noun'
    | 'bs'
    | 'profession'
    | 'industry'
    | 'city_prefix'
    | 'city_suffix'
    | 'city_name'
    | 'country_name'
    | 'country_code'
    | 'street_suffix'
    | 'street_name'
    | 'time_zone'
    | 'state_name'
    | 'state_abbr'
    | 'secondary_address_type'
    | 'secondary_address'
    | 'zip_code'
    | 'post_code'
    | 'building_number'
    | 'latitude'
    | 'longitude'
    | 'phone_number'
    | 'cell_number'
    | 'file_path'
    | 'file_name'
    | 'file_extension'
    | 'dir_path'
    | 'ascii_email'
    | 'ascii_free_email'
    | 'ascii_safe_email'
    | 'address'

interface Faker extends String {
    faker: {
        generator: FakerGenerator
    }
}

interface Categorical extends String {
    categorical: Record<string, number>
}

const str = {
  pattern: function(pattern: RegExp): Pattern {
    return {
      type: 'string',
      pattern: pattern.source,
    };
  },

  format: function(format: string, args: Record<string, Content>): Format {
    return {
      type: 'string',
      format: {
        format,
        arguments: args,
      },
    };
  },

  faker: function(generator: FakerGenerator): Faker {
    return {
      type: 'string',
      faker: {
        generator,
      },
    };
  },

  categorical: function(categorical: Record<string, number>): Categorical {
    return {
      type: 'string',
      categorical,
    };
  },
};

export {str, StringContent};
