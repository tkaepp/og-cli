use clap::{Args, Subcommand, ValueEnum};
use eyre::Result;
use json::object;
use json_to_table::json_to_table;
use serde_json::Value;

use crate::get_config;

pub struct Search;

#[derive(Clone, Debug, ValueEnum)]
pub enum Language {
    LanguageDe,
    LanguageEn,
    LanguageFr,
    LanguageIt,
    LanguageNl,
}

impl Language {
    pub fn get_language_code(&self) -> &str {
        match self {
            Language::LanguageDe => "de-CH",
            Language::LanguageEn => "en-US",
            Language::LanguageFr => "fr-CH",
            Language::LanguageIt => "it-CH",
            Language::LanguageNl => "de-CH",
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Portal {
    PortalChGalaxus,
    PortalChDigitec,
    PortalDe,
    PortalIt,
    PortalFr,
    PortalNl,
    PortalBe,
    PortalAt,
}

impl Portal {
    pub fn get_portal_id(&self) -> i8 {
        match self {
            Portal::PortalChGalaxus => 22,
            Portal::PortalChDigitec => 25,
            Portal::PortalDe => 27,
            Portal::PortalAt => 28,
            Portal::PortalIt => 35,
            Portal::PortalFr => 32,
            Portal::PortalNl => 33,
            Portal::PortalBe => 34,
        }
    }
}

#[derive(Args, Debug)]
pub struct SearchCommand {
    #[command(subcommand)]
    command: SearchSubcommands,
}

#[derive(Subcommand, Debug)]
enum SearchSubcommands {
    ApiSearch {
        environment: SearchEnvironment,
        search_terms: String,
        portal: Option<Portal>,
        language: Option<Language>,
        ltr: Option<Ltr>,
        test_group: Option<String>,
        skip: Option<u8>,
        only_show_visible: Option<bool>,
        group_variants: Option<bool>,
        redirection_take: Option<i8>,
        sort_order: Option<String>,
        take: Option<u8>,
        rewriters: Option<Vec<String>>,
    },
    EsSearch {
        environment: SearchEnvironment,
        query: String,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum SearchEnvironment {
    Oft,
    Test,
    Prod,
}

#[derive(Clone, Debug, ValueEnum)]
enum Ltr {
    LtrOn,
    LtrOff,
}

impl Search {
    pub async fn run(search_command: SearchCommand) -> Result<()> {
        match search_command.command {
            SearchSubcommands::ApiSearch {
                environment,
                search_terms,
                portal,
                language,
                ltr,
                test_group,
                skip,
                only_show_visible,
                group_variants,
                redirection_take,
                sort_order,
                take,
                rewriters,
            } => {
                Self.call_api(
                    match environment {
                        SearchEnvironment::Oft => get_config().search_urls.oft.to_string(),
                        SearchEnvironment::Test => get_config().search_urls.test.to_string(),
                        SearchEnvironment::Prod => get_config().search_urls.prod.to_string(),
                    },
                    search_terms,
                    portal,
                    language,
                    ltr,
                    test_group,
                    skip,
                    only_show_visible,
                    group_variants,
                    redirection_take,
                    sort_order,
                    take,
                    rewriters,
                )
                .await
            }
            SearchSubcommands::EsSearch {
                environment: _,
                query: _,
            } => todo!(),
        }
    }

    async fn call_api(
        &self,
        url: String,
        search_terms: String,
        portal: Option<Portal>,
        language: Option<Language>,
        ltr: Option<Ltr>,
        test_group: Option<String>,
        skip: Option<u8>,
        only_show_visible: Option<bool>,
        group_variants: Option<bool>,
        redirection_take: Option<i8>,
        sort_order: Option<String>,
        take: Option<u8>,
        rewriters: Option<Vec<String>>,
    ) -> Result<()> {
        let portal_id: i8 = match portal {
            None => 22,
            Some(portal) => portal.get_portal_id(),
        };

        let language_code = match &language {
            None => "de-CH".to_string(),
            Some(lang) => lang.get_language_code().to_string(),
        };

        let ltr = match ltr {
            None => "False",
            Some(ltr) => match ltr {
                Ltr::LtrOff => "False",
                Ltr::LtrOn => "True",
            },
        };

        let take_amount: u8 = match take {
            None => 10,
            Some(take) => take,
        };

        let redirection_take_amount: i8 = match redirection_take {
            None => 1,
            Some(redirection_take) => redirection_take,
        };

        let skip_amount: u8 = match skip {
            None => 10,
            Some(skip) => skip,
        };

        let sort_order_str = match sort_order {
            None => "".to_string(),
            Some(sort_order) => sort_order,
        };

        let test_group_str = match test_group {
            None => "".to_string(),
            Some(test_group) => test_group,
        };

        let only_show_visible_bool = match only_show_visible {
            None => false,
            Some(only_show_visible) => only_show_visible,
        };

        let group_variants_bool = match group_variants {
            None => false,
            Some(group_variants) => group_variants,
        };

        let rewriters_str = match rewriters {
            None => "common_rules_ruleset, replace_rules_ruleset".to_string(),
            Some(rewriter_vectors) => rewriter_vectors.join(",").to_string(),
        };

        let request_body = object! {
            "searchTerm": search_terms,
            "portalId": portal_id,
            "languageTag": language_code,
            "product": {"onlyShowVisible": only_show_visible_bool,"groupVariants": group_variants_bool,"take": take_amount,"skip": skip_amount
            },
            "redirection": {"take": redirection_take_amount
            },
            "sortOrder": sort_order_str,
            "searchQueryId": "123456"
        };

        let client = reqwest::Client::new();
        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-DG-TestGroup", test_group_str)
            .header("X-DG-LtrEnabled", ltr)
            .header("X-DG-Rewriters", rewriters_str)
            .body(json::stringify(request_body))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await?;
        println!("{}", json_to_table(&res).to_string());

        // match res {
        //     //Ok(success) => println!("{}", success),
        //     //Ok(success) => println!("{}", serde_json::json!(&success)),
        //     Ok(success) => println!("{}", json_to_table(&serde_json::json!(&success))),
        //     Err(error) => print!("{}", error.to_string()),
        // }
        Ok(())
    }
}
