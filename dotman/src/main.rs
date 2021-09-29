use std::collections::HashMap;

use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand
}

#[derive(Clap)]
enum Subcommand {
    Deploy(DeployOpts),
    DryRun(DryRunOpts)
}

#[derive(Clap)]
struct DeployOpts {
    #[clap(short, long)]
    config: String,
}

#[derive(Clap)]
struct DryRunOpts {
    #[clap(short, long)]
    config: String,
}

enum Task {
    Cp {
        src: String,
        dest: String,
        merge: bool,
    }
}

enum TargetMatcher {
    HostName(String),
}

struct Scenario {
    tasks: Vec<String>,
    matches: Vec<TargetMatcher>,
}

struct PlayBook {
    tasks: HashMap<String, Vec<Task>>,
    scenarios: HashMap<String, Scenario>,
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
        },
        Subcommand::DryRun(opts) => {
        }
    }
}
