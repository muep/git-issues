use clap::{App, AppSettings, Arg};
use regex::Regex;
use serde::Deserialize;
use simple_config_parser::Config;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct IssueResponseFields {
    summary: String,
}

#[derive(Debug, Deserialize)]
struct IssueResponse {
    fields: IssueResponseFields,
}

const SC_ISSUE_SUMMARY: &str = "jira-issue-summary";
const SC_ISSUES_FROM_STDIN: &str = "issues-from-stdin";
const SC_CHECK_JIRA: &str = "check-jira";

async fn check_jira(url: &str, user: &str, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let body = client
        .get(format!("{}/rest/api/3/project/search", url))
        .basic_auth(user, Some(token))
        .send()
        .await?
        .text()
        .await?;

    println!("{}", body);

    Ok(())
}

async fn issue_summary(
    url: &str,
    user: &str,
    token: &str,
    issue: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/rest/api/3/issue/{}", url, issue))
        .basic_auth(user, Some(token))
        .send()
        .await?
        .json::<IssueResponse>()
        .await?;

    Ok(response.fields.summary)
}

fn issues_from_text<'a>(
    prefix: &str,
    text: &'a str,
) -> Result<HashSet<(u32, &'a str)>, Box<dyn std::error::Error>> {
    let rx = Regex::new(&format!(r"{}(?P<id>\d+)", prefix))?;

    Ok(rx
        .find_iter(text)
        .map(|m| {
            let s = m.as_str();
            let caps = rx.captures(s).unwrap();
            let id: u32 = caps["id"].parse().unwrap();
            (id, s)
        })
        .collect())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_arg = Arg::new("config")
        .short('c')
        .long("config")
        .takes_value(true)
        .value_name("CONFIG")
        .help("Path to config file")
        .required(false);

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Produce issue lists from git logs")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(config_arg)
        .subcommand(App::new(SC_CHECK_JIRA).about("Check connectivity with Jira"))
        .subcommand(
            App::new(SC_ISSUES_FROM_STDIN)
                .about("Collect and print issue references from stdin")
                .arg(
                    Arg::new("issue-prefix")
                        .value_name("PREFIX")
                        .help("Prefix before issue number")
                        .required(true),
                ),
        )
        .subcommand(
            App::new(SC_ISSUE_SUMMARY)
                .about("Get an issue summary from Jira")
                .arg(
                    Arg::new("issue")
                        .value_name("ISSUE")
                        .help("Jira issue")
                        .required(true),
                ),
        )
        .get_matches();

    let config = match matches.value_of("config") {
        Some(config_path) => Config::new().file(config_path).unwrap(),
        None => Config::new(),
    };

    match matches.subcommand() {
        Some((SC_CHECK_JIRA, _)) => {
            let token = config.get_str("jira-token").unwrap();
            check_jira(
                &config.get_str("jira-url").unwrap(),
                &config.get_str("jira-user").unwrap(),
                token.trim(),
            )
            .await
        }
        Some((SC_ISSUE_SUMMARY, sub_matches)) => {
            let token = config.get_str("jira-token").unwrap();
            let issue = sub_matches.value_of("issue").unwrap();
            let summary = issue_summary(
                &config.get_str("jira-url").unwrap(),
                &config.get_str("jira-user").unwrap(),
                token.trim(),
                issue,
            )
            .await?;

            println!("{} {}", issue, summary);

            Ok(())
        }
        Some((SC_ISSUES_FROM_STDIN, sub_matches)) => {
            use std::io::Read;

            let prefix = sub_matches.value_of("issue-prefix").unwrap();
            let input = {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            };

            let issues: Vec<&str> = {
                let mut issues: Vec<(u32, &str)> =
                    issues_from_text(&prefix, &input)?.into_iter().collect();

                issues.sort_by_key(|(id, _)| *id);
                issues.into_iter().map(|(_, issue)| issue).collect()
            };

            for issue in issues {
                println!("{}", issue);
            }

            Ok(())
        }
        _ => panic!(
            "subcommand {} is not implemented",
            matches.subcommand_name().unwrap()
        ),
    }
}
