use clap::{AppSettings, Clap};
use std::collections::HashMap;
use std::path::Path;
use std::process;
use termion::color;
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Clap)]
enum Subcommand {
    Deploy(DeployOpts),
    DryRun(DryRunOpts),
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

type TaskGroups<'a> = &'a [(&'a str, &'a [Box<dyn dotman::Task>])];

fn execute_graphicaly(taskgroups: TaskGroups, ctx: &dotman::TaskContext) {
    for (group, tasks) in taskgroups {
        println!("[{}]", group);
        for task in *tasks {
            let task_name = task.name();
            let result = task.execute(ctx);
            match result {
                Ok(true) => println!(
                    "{}[Changed] {}{}",
                    color::Fg(color::Yellow),
                    color::Fg(color::White),
                    task_name
                ),
                Ok(false) => println!(
                    "{}[Ok]      {}{}",
                    color::Fg(color::Green),
                    color::Fg(color::LightWhite),
                    task_name
                ),
                Err(dotman::TaskError::WellKnown(msg)) => {
                    println!(
                        "{}[Failed]  {}{}",
                        color::Fg(color::Red),
                        color::Fg(color::Reset),
                        task_name
                    );
                    println!("  -> {}", msg);
                }
                Err(dotman::TaskError::Unknown(e)) => {
                    println!(
                        "{}[Failed]  {}{}",
                        color::Fg(color::Red),
                        color::Fg(color::Reset),
                        task_name
                    );
                    println!("  -> {}", e);
                }
            }
        }
    }
}

fn run(opts: Opts) -> Result<(), dotman::Error> {
    let mut taskbuilders = HashMap::new();
    let cp_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::cp::parse(yaml));
    let env_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::env::parse(yaml));
    let sh_builder: dotman::TaskBuilder = Box::new(move |yaml| dotman::tasks::sh::parse(yaml));
    let cargo_builder: dotman::TaskBuilder =
        Box::new(move |yaml| dotman::tasks::cargo::parse(yaml));
    taskbuilders.insert("cp".to_owned(), cp_builder);
    taskbuilders.insert("env".to_owned(), env_builder);
    taskbuilders.insert("sh".to_owned(), sh_builder);
    taskbuilders.insert("cargo".to_owned(), cargo_builder);
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let playbook = dotman::PlayBook::load_config(&opts.config, taskbuilders)?;
            let (scenario, deploys) = playbook.deploys()?;
            execute_graphicaly(
                &deploys,
                &dotman::TaskContext {
                    base: Path::new(&opts.config).parent().unwrap().to_owned(),
                    dryrun: false,
                    scenario,
                },
            );
            Ok(())
        }
        Subcommand::DryRun(opts) => {
            let playbook = dotman::PlayBook::load_config(&opts.config, taskbuilders)?;
            let (scenario, deploys) = playbook.deploys()?;
            execute_graphicaly(
                &deploys,
                &dotman::TaskContext {
                    base: Path::new(&opts.config).parent().unwrap().to_owned(),
                    dryrun: true,
                    scenario,
                },
            );
            Ok(())
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
    }
}
