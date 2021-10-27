use clap::Parser;
use dotman::VerboseLevel;
use maplit::hashmap;
use std::process;
use termion::color;
#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Parser)]
enum Subcommand {
    Deploy(DeployOpts),
    DryRun(DryRunOpts),
}

#[derive(Parser)]
struct DeployOpts {
    #[clap(short, long)]
    config: String,
    #[clap(short, long)]
    scenario: Option<String>,
    #[clap(short = 'V', long)]
    verbose: bool,
}

#[derive(Parser)]
struct DryRunOpts {
    #[clap(short, long)]
    config: String,
    #[clap(short, long)]
    scenario: Option<String>,
    #[clap(short = 'V', long)]
    verbose: bool,
}

fn run(opts: Opts) -> Result<(), dotman::Error> {
    let cp_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::cp::parse(yaml));
    let env_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::env::parse(yaml));
    let sh_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::sh::parse(yaml));
    let cargo_builder: dotman::TaskBuilder =
        Box::new(move |yaml| dotman::tasks::cargo::parse(yaml));
    #[cfg(feature = "network")]
    let wget_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::wget::parse(yaml));
    let link_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::link::parse(yaml));

    #[cfg(feature = "network")]
    let taskbuilders = hashmap! {
        "cp".to_owned() => cp_builder,
        "env".to_owned() => env_builder,
        "sh".to_owned() => sh_builder,
        "cargo".to_owned() => cargo_builder,
        "wget".to_owned() => wget_builder,
        "link".to_owned() => link_builder,
    };

    #[cfg(not(feature = "network"))]
    let taskbuilders = hashmap! {
        "cp".to_owned() => cp_builder,
        "env".to_owned() => env_builder,
        "sh".to_owned() => sh_builder,
        "cargo".to_owned() => cargo_builder,
        "link".to_owned() => link_builder,
    };

    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let playbook = dotman::PlayBook::load_config(&opts.config, taskbuilders)?;
            let verbose_lebel = if opts.verbose {
                VerboseLevel::ShowAllTask
            } else {
                VerboseLevel::Compact
            };
            if let Some(scenario) = opts.scenario {
                playbook.execute_graphicaly(false, Some(&scenario), &verbose_lebel)
            } else {
                playbook.execute_graphicaly(false, None, &verbose_lebel)
            }
        }
        Subcommand::DryRun(opts) => {
            let playbook = dotman::PlayBook::load_config(&opts.config, taskbuilders)?;
            let verbose_lebel = if opts.verbose {
                VerboseLevel::ShowAllTask
            } else {
                VerboseLevel::Compact
            };
            if let Some(scenario) = opts.scenario {
                playbook.execute_graphicaly(true, Some(&scenario), &verbose_lebel)
            } else {
                playbook.execute_graphicaly(true, None, &verbose_lebel)
            }
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match run(opts) {
        Ok(()) => (),
        Err(dotman::Error::AnyScenarioDoesNotMatch) => {
            eprintln!(
                "{}[Error] {}any scenario does not match",
                color::Fg(color::Red),
                color::Fg(color::Reset)
            );
            process::exit(-1);
        }
        Err(dotman::Error::TaskGroupNotFound(taskgroup_name)) => {
            eprintln!(
                "{}[Error] {}taskgroup \"{}\" does not found",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                taskgroup_name
            );
            process::exit(-1);
        }
        Err(dotman::Error::PlaybookLoadFailed(msg)) => {
            eprintln!(
                "{}[Error] {}failed to load playbook due to {}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                msg
            );
            process::exit(-1);
        }
        Err(dotman::Error::InvalidPlaybook(msg, _)) => {
            eprintln!(
                "{}[Error] {}failed to load playbook due to {}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                msg
            );
            process::exit(-1);
        }
        Err(dotman::Error::CannotResolveVar(var, e)) => {
            eprintln!(
                "{}[Error] {}cannot resolve var ${} due to {:?}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                var,
                e
            );
            process::exit(-1);
        }
        Err(dotman::Error::CannotCollectNodeInformation(msg)) => {
            eprintln!(
                "{}[Error] {}cannot collect node information due to {}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                msg
            )
        }
        Err(dotman::Error::UnrecognizedMembers { prefix, members }) => {
            if let Some(prefix) = prefix {
                for (key, _) in members {
                    eprintln!(
                        "{}[Error] {}unrecognized member {}.{}",
                        color::Fg(color::Red),
                        color::Fg(color::Reset),
                        prefix,
                        key
                    );
                }
            } else {
                for (key, _) in members {
                    eprintln!(
                        "{}[Error] {}unrecognized member {}",
                        color::Fg(color::Red),
                        color::Fg(color::Reset),
                        key
                    );
                }
            }
        }
    }
}
