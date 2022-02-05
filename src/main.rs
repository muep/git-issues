use clap::{App, AppSettings, Arg};
use serde::Deserialize;
use simple_config_parser::Config;

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
    let config_arg = Arg::new("config")
        .short('c')
        .long("config")
        .takes_value(true)
        .value_name("CONFIG")
        .help("Path to config file")
        .required(true);

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Produce issue lists from git logs")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(config_arg)
        .subcommand(App::new(SC_CHECK_JIRA).about("Check connectivity with Jira"))
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

    let config = {
        let config_path = matches.value_of("config").unwrap();
        Config::new().file(config_path).unwrap()
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
        _ => Ok(()),
    }
}
