use super::super::prelude::*;
use fake::faker::address::raw::*;
use fake::faker::company::raw::*;
use fake::faker::creditcard::raw::CreditCardNumber;
use fake::faker::filesystem::raw::*;
use fake::faker::http::raw::*;
use fake::faker::internet::raw::*;
use fake::faker::lorem::raw::*;
use fake::faker::name::raw::*;
use fake::faker::phone_number::raw::*;
use fake::{locales, Fake};
use rand::RngCore;

// this needs non-camel-case types because the fake crate has the same
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Hash)]
/// a locale to look up names, addresses, etc.
pub enum Locale {
    EN,
    FR_FR,
    ZH_TW,
    ZH_CN,
}

impl Default for Locale {
    fn default() -> Self {
        Self::EN
    }
}

/// The arguments for a faker
#[derive(Clone, Default, Deserialize, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct FakerArgs {
    #[serde(default)]
    pub(crate) locales: Vec<Locale>,
}

type FakerFunction = for<'r> fn(&'r mut dyn RngCore, &FakerArgs) -> String;

macro_rules! fake_map_entry {
    (locale; $name:ident, $rng:ident, $args:ident, $map:ident, $faker:ident) => {
        fake_map_entry!(with_locales; $name, $rng, $args, $map, $faker; EN, FR_FR, ZH_TW, ZH_CN)
    };
    (with_locales; $name:ident, $rng:ident, $args:ident, $map:ident, $faker:ident; $($locale:ident),*) => {
        fn $name($rng: &mut dyn RngCore, $args: &FakerArgs) -> String {
            match $args.locales.get($rng.gen_range(0..$args.locales.len().max(1))).unwrap_or(&Locale::EN) {
                $(Locale::$locale => $faker(locales::$locale).fake_with_rng($rng)),*
            }
        }
        $map.insert(stringify!($name), $name as _);
    };
    ($name:ident, $rng:ident, $args:ident, $map:ident, $e:expr) => {
        fn $name($rng: &mut dyn RngCore, $args: &FakerArgs) -> String {
            $e
        }
        $map.insert(stringify!($name), $name as _);
    };
}

lazy_static! {
    static ref FAKE_MAP: HashMap<&'static str, FakerFunction> = {
        let mut m = HashMap::new();
        // Lorem

        fake_map_entry!(locale; word, r, args, m, Word);
        // TODO here need to pass some params
        // Words(count: Range<usize>);
        // Sentence(count: Range<usize>);
        // Sentences(count: Range<usize>);
        // Paragraph(count: Range<usize>);
        // Paragraphs(count: Range<usize>);

        // Name
        fake_map_entry!(locale; first_name, r, args, m, FirstName);
        fake_map_entry!(locale; last_name, r, args, m, LastName);
        fake_map_entry!(locale; title, r, args, m, Title);
        fake_map_entry!(locale; suffix, r, args, m, Suffix);
        fake_map_entry!(locale; name, r, args, m, Name);
        fake_map_entry!(locale; name_with_title, r, args, m, NameWithTitle);

        // Credit Card Number
        fake_map_entry!(locale; credit_card, r, args, m, CreditCardNumber);

        // Internet
        fake_map_entry!(locale; free_email_provider, r, args, m, FreeEmailProvider);
        fake_map_entry!(locale; domain_suffix, r, args, m, DomainSuffix);
        fake_map_entry!(locale; free_email, r, args, m, FreeEmail);
        fake_map_entry!(locale; safe_email, r, args, m, SafeEmail);
        fake_map_entry!(locale; username, r, args, m, Username);
        fake_map_entry!(locale; ipv4, r, args, m, IPv4);
        fake_map_entry!(locale; ipv6, r, args, m, IPv6);
        fake_map_entry!(locale; ip, r, args, m, IP);
        fake_map_entry!(locale; mac_address, r, args, m, MACAddress);
        fake_map_entry!(locale; color, r, args, m, Color);
        fake_map_entry!(locale; user_agent, r, args, m, UserAgent);

        // HTTP
        fake_map_entry!(locale; rfc_status_code, r, args, m, RfcStatusCode);
        fake_map_entry!(locale; valid_status_code, r, args, m, ValidStatusCode);

        // Company
        fake_map_entry!(locale; company_suffix, r, args, m, CompanySuffix);
        fake_map_entry!(locale; company_name, r, args, m, CompanyName);
        fake_map_entry!(locale; buzzword, r, args, m, Buzzword);
        fake_map_entry!(locale; buzzword_muddle, r, args, m, BuzzwordMiddle);
        fake_map_entry!(locale; buzzword_tail, r, args, m, BuzzwordTail);
        fake_map_entry!(locale; catch_phrase, r, args, m, CatchPhase);
        fake_map_entry!(locale; bs_verb, r, args, m, BsVerb);
        fake_map_entry!(locale; bs_adj, r, args, m, BsAdj);
        fake_map_entry!(locale; bs_noun, r, args, m, BsNoun);
        fake_map_entry!(locale; bs, r, args, m, Bs);
        fake_map_entry!(locale; profession, r, args, m, Profession);
        fake_map_entry!(locale; industry, r, args, m, Industry);

        // Address
        fake_map_entry!(locale; city_prefix, r, args, m, CityPrefix);
        fake_map_entry!(locale; city_suffix, r, args, m, CitySuffix);
        fake_map_entry!(locale; city_name, r, args, m, CityName);
        fake_map_entry!(locale; country_name, r, args, m, CountryName);
        fake_map_entry!(locale; country_code, r, args, m, CountryCode);
        fake_map_entry!(locale; street_suffix, r, args, m, StreetSuffix);
        fake_map_entry!(locale; street_name, r, args, m, StreetName);
        fake_map_entry!(locale; time_zone, r, args, m, TimeZone);
        fake_map_entry!(locale; state_name, r, args, m, StateName);
        fake_map_entry!(locale; state_abbr, r, args, m, StateAbbr);
        fake_map_entry!(locale; secondary_address_type, r, args, m, SecondaryAddressType);
        fake_map_entry!(locale; secondary_address, r, args, m, SecondaryAddress);
        fake_map_entry!(locale; zip_code, r, args, m, ZipCode);
        fake_map_entry!(locale; post_code, r, args, m, PostCode);
        fake_map_entry!(locale; building_number, r, args, m, BuildingNumber);
        fake_map_entry!(locale; latitude, r, args, m, Latitude);
        fake_map_entry!(locale; longitude, r, args, m, Longitude);

        // Phone Number
        fake_map_entry!(locale; phone_number, r, args, m, PhoneNumber);
        fake_map_entry!(locale; cell_number, r, args, m, CellNumber);

        // FileSystem
        fake_map_entry!(locale; file_path, r, args, m, FilePath);
        fake_map_entry!(locale; file_name, r, args, m, FileName);
        fake_map_entry!(locale; file_extension, r, args, m, FileExtension);
        fake_map_entry!(locale; dir_path, r, args, m, DirPath);

        // Aliases
        fake_map_entry!(locale; ascii_email, r, args, m, SafeEmail);
        fake_map_entry!(locale; ascii_free_email, r, args, m, FreeEmail);
        fake_map_entry!(locale; ascii_safe_email, r, args, m, SafeEmail);

        // Custom
        m.insert("address", address as _);

        m
    };
}

fn address(rng: &mut dyn RngCore, args: &FakerArgs) -> String {
    // Here we get a single locale
    let args = &FakerArgs {
        locales: vec![*args.locales.first().unwrap_or(&Locale::EN)],
    };

    let number = (FAKE_MAP.get("building_number").unwrap())(rng, args);
    let street_name = (FAKE_MAP.get("street_name").unwrap())(rng, args);
    let state_abbr = (FAKE_MAP.get("state_abbr").unwrap())(rng, args);
    let zip_code = (FAKE_MAP.get("zip_code").unwrap())(rng, args);
    format!("{} {}, {} {}", number, street_name, state_abbr, zip_code)
}

pub struct RandFaker {
    generator: FakerFunction,
    args: FakerArgs,
}

impl RandFaker {
    pub(crate) fn new<S: AsRef<str>>(generator: S, args: FakerArgs) -> Result<Self, anyhow::Error> {
        match FAKE_MAP.get(generator.as_ref()) {
            None => Err(anyhow!(
                "Generator '{}' does not exist {}",
                generator.as_ref(),
                suggest_closest(FAKE_MAP.keys(), generator.as_ref())
                    .unwrap_or_else(|| "".to_string())
            )),
            Some(generator) => Ok(Self {
                generator: *generator,
                args,
            }),
        }
    }
}

impl Generator for RandFaker {
    type Yield = String;

    type Return = Result<Never, Error>;

    fn next<R: Rng>(&mut self, rng: &mut R) -> GeneratorState<Self::Yield, Self::Return> {
        GeneratorState::Yielded((self.generator)(rng, &self.args))
    }
}
