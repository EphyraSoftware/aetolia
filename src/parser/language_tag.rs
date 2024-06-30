//! https://www.rfc-editor.org/rfc/rfc5646.txt

use crate::parser::Error;
use nom::branch::alt;
use nom::bytes::streaming::{tag, take_while_m_n};
use nom::character::streaming::char;
use nom::character::{is_alphabetic, is_alphanumeric, is_digit};
use nom::combinator::{opt, peek, recognize, verify};
use nom::error::ParseError;
use nom::multi::{many0, many1, many_m_n};
use nom::sequence::tuple;
use nom::{IResult, Parser};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageTag {
    pub language: String,
    pub ext_lang: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub variants: Vec<String>,
    pub extensions: Vec<String>,
    pub private_use: Option<String>,
}

#[cfg(test)]
impl Default for LanguageTag {
    fn default() -> Self {
        Self {
            language: String::new(),
            ext_lang: None,
            script: None,
            region: None,
            variants: Vec::with_capacity(0),
            extensions: Vec::with_capacity(0),
            private_use: None,
        }
    }
}

#[inline]
const fn is_singleton(b: u8) -> bool {
    matches!(b, b'\x30'..=b'\x39' | b'\x41'..=b'\x57' | b'\x59'..=b'\x5A' | b'\x61'..=b'\x77' | b'\x79'..=b'\x7A')
}

fn private_use<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    recognize(tuple((
        char('x'),
        many1(tuple((char('-'), take_while_m_n(1, 8, is_alphanumeric)))),
    )))(input)
}

pub fn language_tag<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], LanguageTag, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, grandfathered_irregular) = opt(alt((
        tag("en-GB-oed"),
        tag("i-ami"),
        tag("i-bnn"),
        tag("i-default"),
        tag("i-enochian"),
        tag("i-hak"),
        tag("i-klingon"),
        tag("i-lux"),
        tag("i-mingo"),
        tag("i-navajo"),
        tag("i-pwn"),
        tag("i-tao"),
        tag("i-tay"),
        tag("i-tsu"),
        tag("sgn-BE-FR"),
        tag("sgn-BE-NL"),
        tag("sgn-CH-DE"),
    )))(input)?;

    if let Some(grandfathered_irregular) = grandfathered_irregular {
        let language_tag = LanguageTag {
            language: String::from_utf8_lossy(grandfathered_irregular).to_string(),
            ext_lang: None,
            script: None,
            region: None,
            variants: Vec::with_capacity(0),
            extensions: Vec::with_capacity(0),
            private_use: None,
        };

        return Ok((input, language_tag));
    }

    let (input, private_use) = opt(private_use)(input)?;
    if let Some(private_use) = private_use {
        let language_tag = LanguageTag {
            language: String::from_utf8_lossy(private_use).to_string(),
            ext_lang: None,
            script: None,
            region: None,
            variants: Vec::with_capacity(0),
            extensions: Vec::with_capacity(0),
            private_use: None,
        };

        return Ok((input, language_tag));
    }

    lang_tag(input)
}

/// Peeks at the next byte and checks that
///   - If the next byte is a `-`, then accept
///   - If the next byte is alphanumeric, then reject
///
/// This can be used to prevent bad matches that end in the middle of a component.
fn clip<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    peek(verify(
        take_while_m_n(0, 1, |c| c == b'-' || is_alphanumeric(c)),
        |m: &[u8]| m == [b'-'] || m.is_empty(),
    ))(input)
}

pub fn lang_tag<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], LanguageTag, E>
where
    E: ParseError<&'a [u8]> + From<Error<'a>>,
{
    let (input, (language, ext_lang)) = alt((
        tuple((
            take_while_m_n(2, 3, is_alphabetic),
            opt(tuple((
                char('-'),
                recognize(tuple((
                    take_while_m_n(3, 3, is_alphabetic),
                    many_m_n(
                        0,
                        2,
                        tuple((char('-'), take_while_m_n(3, 3, is_alphabetic), clip)),
                    ),
                    clip,
                ))),
            ))),
        )),
        take_while_m_n(4, 4, is_alphabetic).map(|l| (l, None)),
        take_while_m_n(5, 8, is_alphabetic).map(|l| (l, None)),
    ))(input)?;

    let mut language_tag = LanguageTag {
        language: String::from_utf8_lossy(language).to_string(),
        ext_lang: ext_lang.map(|(_, ext_lang)| String::from_utf8_lossy(ext_lang).to_string()),
        script: None,
        region: None,
        variants: Vec::with_capacity(0),
        extensions: Vec::with_capacity(0),
        private_use: None,
    };

    // Find the script, if present
    let (input, script) = opt(tuple((
        char('-'),
        take_while_m_n(4, 4, is_alphabetic),
        clip,
    )))(input)?;

    if let Some((_, script, _)) = script {
        language_tag.script = Some(String::from_utf8_lossy(script).to_string());
    }

    // Find the region, if present
    let (input, region) = opt(tuple((
        char('-'),
        alt((
            tuple((take_while_m_n(2, 2, is_alphabetic), clip)),
            tuple((take_while_m_n(3, 3, is_digit), clip)),
        )),
    )))(input)?;

    if let Some((_, (region, _))) = region {
        language_tag.region = Some(String::from_utf8_lossy(region).to_string());
    }

    // Find variants, is present
    let (input, variants) = many0(tuple((
        char('-'),
        alt((
            take_while_m_n(5, 8, is_alphanumeric),
            recognize(tuple((
                take_while_m_n(1, 1, is_digit),
                take_while_m_n(3, 3, is_alphanumeric),
            ))),
        )),
    )))(input)?;

    if !variants.is_empty() {
        language_tag.variants = variants
            .into_iter()
            .map(|(_, v)| String::from_utf8_lossy(v).to_string())
            .collect();
    }

    // Find extensions, if present
    let (input, extensions) = many0(tuple((
        char('-'),
        recognize(tuple((
            take_while_m_n(1, 1, is_singleton),
            many1(tuple((char('-'), take_while_m_n(2, 8, is_alphanumeric)))),
        ))),
    )))(input)?;

    if !extensions.is_empty() {
        language_tag.extensions = extensions
            .into_iter()
            .map(|(_, ext)| String::from_utf8_lossy(ext).to_string())
            .collect();
    }

    // Find private use, if present
    let (input, private_use) = opt(tuple((char('-'), private_use)))(input)?;

    if let Some((_, private_use)) = private_use {
        language_tag.private_use = Some(String::from_utf8_lossy(private_use).to_string());
    }

    Ok((input, language_tag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::check_rem;
    use test_case::test_case;

    #[test_case(b"de;"; "German")]
    #[test_case(b"fr;"; "French")]
    #[test_case(b"ja;"; "Japanese")]
    #[test_case(b"i-enochian;"; "example of a grandfathered tag")]
    fn simple_lang_subtag(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);
        assert_eq!(
            &input[..(input.len() - 1)],
            language_tag.language.as_bytes()
        );

        assert!(language_tag.ext_lang.is_none());
        assert!(language_tag.script.is_none());
        assert!(language_tag.region.is_none());
        assert!(language_tag.variants.is_empty());
        assert!(language_tag.extensions.is_empty());
        assert!(language_tag.private_use.is_none());
    }

    #[test_case(b"zh-Hant;"; "Chinese written using the Traditional Chinese script")]
    #[test_case(b"zh-Hans;"; "Chinese written using the Simplified Chinese script")]
    #[test_case(b"sr-Cyrl;"; "Serbian written using the Cyrillic script")]
    #[test_case(b"sr-Latn;"; "Serbian written using the Latin script")]
    fn language_subtag_plug_script_subtag(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        let str = String::from_utf8(input[..(input.len() - 1)].to_vec()).unwrap();
        assert_eq!(
            str.split('-').next().unwrap().as_bytes(),
            language_tag.language.as_bytes()
        );
        assert_eq!(
            Some(str.split('-').nth(1).unwrap().to_string()),
            language_tag.script
        );

        assert!(language_tag.ext_lang.is_none());
        assert!(language_tag.region.is_none());
        assert!(language_tag.variants.is_empty());
        assert!(language_tag.extensions.is_empty());
        assert!(language_tag.private_use.is_none());
    }

    #[test_case(b"zh-cmn-Hans-CN;"; "Chinese, Mandarin, Simplified script, as used in China")]
    #[test_case(b"cmn-Hans-CN;"; "Mandarin Chinese, Simplified script, as used in China")]
    #[test_case(b"zh-yue-HK;"; "Chinese, Cantonese, as used in Hong Kong SAR")]
    #[test_case(b"yue-HK;"; "Cantonese Chinese, as used in Hong Kong SAR")]
    fn extended_language_subtags_and_their_primary_language_subtag_counterparts(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language,
                ext_lang,
                script,
                region,
                ..
            } if language == "zh" => match region.unwrap().as_str() {
                "CN" => {
                    assert_eq!(Some("cmn".to_string()), ext_lang);
                    assert_eq!(Some("Hans".to_string()), script);
                }
                "HK" => {
                    assert_eq!(Some("yue".to_string()), ext_lang);
                }
                _ => panic!("Unexpected region"),
            },
            LanguageTag {
                language,
                script,
                region,
                ..
            } if language == "cmn" => {
                assert_eq!(Some("Hans".to_string()), script);
                assert_eq!(Some("CN".to_string()), region);
            }
            LanguageTag {
                language, region, ..
            } if language == "yue" => {
                assert_eq!(Some("HK".to_string()), region);
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"zh-Hans-CN;"; "Chinese written using the Simplified script as used in mainland China")]
    #[test_case(b"sr-Latn-RS;"; "Serbian written using the Latin script as used in Serbia")]
    fn language_script_region(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language,
                script,
                region,
                ..
            } if language == "zh" => {
                assert_eq!(Some("Hans".to_string()), script);
                assert_eq!(Some("CN".to_string()), region);
            }
            LanguageTag {
                language,
                script,
                region,
                ..
            } if language == "sr" => {
                assert_eq!(Some("Latn".to_string()), script);
                assert_eq!(Some("RS".to_string()), region);
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"sl-rozaj;"; "Resian dialect of Slovenian")]
    #[test_case(b"sl-rozaj-biske;"; "San Giorgio dialect of Resian dialect of Slovenian")]
    #[test_case(b"sl-nedis;"; "Nadiza dialect of Slovenian")]
    fn language_variant(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language, variants, ..
            } if language == "sl" => {
                match variants.first().unwrap().as_str() {
                    "rozaj" => {
                        if variants.len() == 1 {
                            // Okay
                        } else if variants.len() == 2 {
                            assert_eq!("biske", variants.last().unwrap().as_str());
                        } else {
                            panic!("Unexpected number of variants")
                        }
                    }
                    "nedis" => {
                        assert_eq!(1, variants.len());
                    }
                    _ => panic!("Unexpected variant"),
                }
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"de-CH-1901;"; "German as used in Switzerland using the 1901 variant [orthography]")]
    #[test_case(b"sl-IT-nedis;"; "Slovenian as used in Italy, Nadiza dialect")]
    fn language_region_variant(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language,
                region,
                variants,
                ..
            } if language == "de" => {
                assert_eq!(Some("CH".to_string()), region);
                assert_eq!(1, variants.len());
                assert_eq!("1901", variants.first().unwrap().as_str());
            }
            LanguageTag {
                language,
                region,
                variants,
                ..
            } if language == "sl" => {
                assert_eq!(Some("IT".to_string()), region);
                assert_eq!(1, variants.len());
                assert_eq!("nedis", variants.first().unwrap().as_str());
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"hy-Latn-IT-arevela;"; "Eastern Armenian written in Latin script, as used in Italy")]
    fn language_script_region_variant(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language,
                script,
                region,
                variants,
                ..
            } if language == "hy" => {
                assert_eq!(Some("Latn".to_string()), script);
                assert_eq!(Some("IT".to_string()), region);
                assert_eq!(1, variants.len());
                assert_eq!("arevela", variants.first().unwrap().as_str());
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"de-DE;"; "German for Germany")]
    #[test_case(b"en-US;"; "English as used in the United States")]
    #[test_case(b"es-419;"; "Spanish appropriate for the Latin America and Caribbean region using the UN region code")]
    fn language_region(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language, region, ..
            } if language == "de" => {
                assert_eq!(Some("DE".to_string()), region);
            }
            LanguageTag {
                language, region, ..
            } if language == "en" => {
                assert_eq!(Some("US".to_string()), region);
            }
            LanguageTag {
                language, region, ..
            } if language == "es" => {
                assert_eq!(Some("419".to_string()), region);
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"de-CH-x-phonebk;"; "phonebk")]
    #[test_case(b"az-Arab-x-AZE-derbend;"; "derbend")]
    fn private_use_subtags(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language,
                region,
                private_use,
                ..
            } if language == "de" => {
                assert_eq!(Some("CH".to_string()), region);
                assert_eq!(Some("x-phonebk".to_string()), private_use);
            }
            LanguageTag {
                language,
                script,
                private_use,
                ..
            } if language == "az" => {
                assert_eq!(Some("Arab".to_string()), script);
                assert_eq!(Some("x-AZE-derbend".to_string()), private_use);
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"x-whatever;"; "private use using the singleton 'x'")]
    #[test_case(b"qaa-Qaaa-QM-x-southern;"; "all private tags")]
    #[test_case(b"de-Qaaa;"; "German, with a private script")]
    #[test_case(b"sr-Latn-QM;"; "Serbian, Latin script, private region")]
    #[test_case(b"sr-Qaaa-RS;"; "Serbian, private script, for Serbia")]
    fn private_use_registry_values(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag { language, .. } if language == "x-whatever" => {
                // Okay
            }
            LanguageTag {
                language,
                script,
                region,
                private_use,
                ..
            } if language == "qaa" => {
                assert_eq!(Some("Qaaa".to_string()), script);
                assert_eq!(Some("QM".to_string()), region);
                assert_eq!(Some("x-southern".to_string()), private_use);
            }
            LanguageTag {
                language, script, ..
            } if language == "de" => {
                assert_eq!(Some("Qaaa".to_string()), script);
            }
            LanguageTag {
                language,
                script,
                region,
                ..
            } if language == "sr" => match script.unwrap().as_str() {
                "Latn" => {
                    assert_eq!(Some("QM".to_string()), region);
                }
                "Qaaa" => {
                    assert_eq!(Some("RS".to_string()), region);
                }
                _ => panic!("Unexpected script"),
            },
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"en-US-u-islamcal;"; "islamcal")]
    #[test_case(b"zh-CN-a-myext-x-private;"; "myext and private")]
    #[test_case(b"en-a-myext-b-another;"; "myext and another")]
    fn tags_that_use_extensions(input: &[u8]) {
        let (rem, language_tag) = language_tag::<Error>(input).unwrap();
        check_rem(rem, 1);

        match language_tag {
            LanguageTag {
                language,
                region,
                extensions,
                ..
            } if language == "en" => {
                if extensions.len() == 1 {
                    assert_eq!(Some("US".to_string()), region);
                    assert_eq!(1, extensions.len());
                    assert_eq!("u-islamcal", extensions.first().unwrap().as_str());
                } else if extensions.len() == 2 {
                    assert_eq!("a-myext", extensions.first().unwrap().as_str());
                    assert_eq!("b-another", extensions.last().unwrap().as_str());
                } else {
                    panic!("Unexpected number of extensions")
                }
            }
            LanguageTag {
                language,
                region,
                extensions,
                private_use,
                ..
            } if language == "zh" => {
                assert_eq!(Some("CN".to_string()), region);
                assert_eq!(1, extensions.len());
                assert_eq!("a-myext", extensions.first().unwrap().as_str());
                assert_eq!(Some("x-private".to_string()), private_use);
            }
            _ => panic!("Unexpected result"),
        }
    }

    #[test_case(b"de-419-DE;"; "two region tags")]
    #[test_case(b"a-DE;"; "use of a single-character subtag in primary position; note that there are a few grandfathered tags that start with \"i-\" that are valid")]
    // This is not a parser failure but a content validation failure -> #[test_case(b"ar-a-aaa-b-bbb-a-ccc;"; "two extensions with same single-letter prefix")]
    fn some_invalid_tags(input: &[u8]) {
        let r = language_tag::<Error>(input);
        match r {
            Err(nom::Err::Error(_)) => {}
            Ok((rem, lang)) => assert!(rem.len() > 1, "Created lang: {lang:?}"),
            r => panic!("Unexpected result: {r:?}"),
        }
    }
}
