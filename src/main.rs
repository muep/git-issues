use std::fs::read_to_string;

use clap::{arg, App, AppSettings};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Produce issue lists from git logs")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new(SC_CHECK_JIRA)
                .about("Check connectivity with Jira")
                .arg(
                    arg!(-t --token <TOKEN> "Path to file with Personal access token")
                        .required(true),
                )
                .arg(arg!(<URL> "Jira URL"))
                .arg(arg!(<USER> "User account (email address) to Jira")),
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
        _ => Ok(()),
    }
}
