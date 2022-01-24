use clap::{App, AppSettings};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Produce issue lists from git logs")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("check-jira").about("Check connectivity with Jira"))
        .get_matches();
    println!("Hello, world!");
}
