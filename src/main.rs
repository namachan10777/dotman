use clap::{AppSettings, Clap};
use std::path::Path;
use std::process;
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

fn run(opts: Opts) -> Result<(), dotman::Error> {
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let playbook = dotman::PlayBook::load_config(&opts.config)?;
            let ctx = dotman::TaskContext {
                base: Path::new(&opts.config).parent().unwrap().to_owned(),
                dryrun: false,
            };
            let result = playbook.execute_deploy(&ctx)?;
            println!("{:#?}", result);
            Ok(())
        }
        Subcommand::DryRun(opts) => {
            let playbook = dotman::PlayBook::load_config(&opts.config)?;
            let ctx = dotman::TaskContext {
                base: Path::new(&opts.config).parent().unwrap().to_owned(),
                dryrun: true,
            };
            let result = playbook.execute_deploy(&ctx)?;
            println!("{:#?}", result);
            Ok(())
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match run(opts) {
        Ok(()) => (),
        Err(dotman::Error::AnyScenarioDoesNotMatch) => {
            eprintln!("any scenario does not match");
            process::exit(-1);
        }
        Err(dotman::Error::TaskGroupNotFound(taskgroup_name)) => {
            eprintln!("taskgroup \"{}\" does not found", taskgroup_name);
            process::exit(-1);
        }
        Err(dotman::Error::PlaybookLoadFailed(msg)) => {
            eprintln!("failed to load playbook due to {}", msg);
            process::exit(-1);
        }
        Err(dotman::Error::InvalidPlaybook(msg, _)) => {
            eprintln!("failed to load playbook due to {}", msg);
            process::exit(-1);
        }
        Err(dotman::Error::CannotResolveVar(var)) => {
            eprintln!("cannot resolve var ${}", var);
            process::exit(-1);
        }
    }
}
