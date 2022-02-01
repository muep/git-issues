use clap::{arg, App, AppSettings, Arg};
use serde::Deserialize;
use std::fs::read_to_string;

#[derive(Debug, Deserialize)]
struct Fields {
    summary: String,
}

#[derive(Debug, Deserialize)]
struct Issue {
    fields: Fields,
}

const SC_ISSUE_SUMMARY: &str = "jira-issue-summary";
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

    let body = client
        .get(format!("{}/rest/api/3/issue/{}", url, issue))
        .basic_auth(user, Some(token))
        .send()
        .await?
        .json::<Issue>()
        .await?;

    Ok(body.fields.summary)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token_arg = Arg::new("token")
        .short('t')
        .long("token")
        .takes_value(true)
        .value_name("TOKEN")
        .help("Path to file with Personal access token")
        .required(true);

    let url_parg = Arg::new("jira-url")
        .value_name("URL")
        .help("Jira URL")
        .required(true);

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Produce issue lists from git logs")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new(SC_CHECK_JIRA)
                .about("Check connectivity with Jira")
                .arg(&token_arg)
                .arg(&url_parg)
                .arg(arg!(<USER> "User account (email address) to Jira")),
        )
        .subcommand(
            App::new(SC_ISSUE_SUMMARY)
                .about("Get an issue summary from Jira")
                .arg(token_arg)
                .arg(url_parg)
                .arg(arg!(<USER> "User account (email address) to Jira"))
                .arg(arg!(<ISSUE> "Jira issue")),
        )
        .get_matches();

    match matches.subcommand() {
        Some((SC_CHECK_JIRA, sub_matches)) => {
            let token = read_to_string(sub_matches.value_of("token").unwrap()).unwrap();
            check_jira(
                sub_matches.value_of("URL").unwrap(),
                sub_matches.value_of("USER").unwrap(),
                token.trim(),
            )
            .await
        }
        Some((SC_ISSUE_SUMMARY, sub_matches)) => {
            let token = read_to_string(sub_matches.value_of("token").unwrap()).unwrap();
            let issue = sub_matches.value_of("ISSUE").unwrap();
            let summary = issue_summary(
                sub_matches.value_of("jira-url").unwrap(),
                sub_matches.value_of("USER").unwrap(),
                token.trim(),
                issue,
            )
            .await?;

            println!("{} {}", issue, summary);

            Ok(())
        }
        _ => Ok(()),
    }
}
