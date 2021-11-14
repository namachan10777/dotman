use clap::Parser;
use dotman::VerboseLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::{fs, io, process};
use termion::color;
#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Parser)]
enum Subcommand {
    #[clap(about = "deploy dotfiles")]
    Deploy(DeployOpts),
    #[clap(about = "dry run")]
    DryRun(DryRunOpts),
    #[clap(about = "generate shell completion")]
    Completion(CompletionOpts),
}

#[derive(Parser)]
struct CompletionOpts {
    #[clap(
        short,
        long,
        possible_value = "fish",
        possible_value = "zsh",
        possible_value = "bash",
        possible_value = "elvish",
        possible_value = "powershell"
    )]
    shell: String,
}

#[derive(Parser)]
struct DeployOpts {
    #[clap(index = 1, about = "specify configuration file e.g. \"dotfiles.yaml\"")]
    config: String,
    #[clap(long = "no-cache", about = "deploy without cache")]
    no_cache: bool,
    #[clap(
        short,
        long,
        about = "specify scenario with no auto scenario detection"
    )]
    #[clap(short = 's', long = "scenario")]
    scenario: Option<String>,
    #[clap(short = 'V', long)]
    verbose: bool,
}

#[derive(Parser)]
struct DryRunOpts {
    #[clap(index = 1, about = "specify configuration file e.g. \"dotfiles.yaml\"")]
    config: String,
    #[clap(long = "no-cache", about = "dry-run without cache")]
    no_cache: bool,
    #[clap(
        short,
        long,
        about = "specify scenario with no auto scenario detection"
    )]
    scenario: Option<String>,
    #[clap(short = 'V', long)]
    verbose: bool,
}

#[derive(Serialize, Deserialize)]
struct Cache {
    cargo: Option<dotman::tasks::cargo::Cache>,
}

struct TaskBuilder {
    ids: Vec<&'static str>,
    cache: Cache,
}

impl dotman::TaskBuilder for TaskBuilder {
    fn parse(
        taskname: &str,
        hash: &HashMap<String, dotman::ast::Value>,
    ) -> Option<Result<dotman::TaskEntity, dotman::Error>> {
        match taskname {
            "cp" => Some(dotman::tasks::cp::parse(hash)),
            "env" => Some(dotman::tasks::env::parse(hash)),
            "sh" => Some(dotman::tasks::sh::parse(hash)),
            "cargo" => Some(dotman::tasks::cargo::parse(hash)),
            "link" => Some(dotman::tasks::link::parse(hash)),
            #[cfg(feature = "network")]
            "wget" => Some(dotman::tasks::wget::parse(hash)),
            _ => None,
        }
    }

    fn cache(&self, key: &str) -> Option<Vec<u8>> {
        match (key, &self.cache.cargo) {
            ("cargo", Some(cache)) => Some(rmp_serde::to_vec(cache).unwrap()),
            _ => None,
        }
    }

    fn ids(&self) -> &[&str] {
        self.ids.as_slice()
    }
}

impl TaskBuilder {
    fn from_cache_path<P: AsRef<Path>>(path: Option<P>) -> Self {
        #[cfg(feature = "network")]
        let ids = vec!["cp", "env", "sh", "cargo", "link", "wget"];
        #[cfg(not(feature = "network"))]
        let ids = vec!["cp", "env", "sh", "cargo", "link"];
        if let Some(path) = path {
            if let Ok(f) = fs::File::open(path) {
                if let Ok(cache) = serde_json::from_reader(f) {
                    return Self { ids, cache };
                }
            }
        };
        Self {
            cache: Cache { cargo: None },
            ids,
        }
    }
}

fn run(opts: Opts) -> Result<(), dotman::Error> {
    let cache_path = format!(
        "{}/.dotfiles.cache.json",
        std::env::var("HOME")
            .map_err(|_| dotman::Error::CannotLoadCache("Cannot expand $HOME".to_owned()))?,
    );
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let task_builder = TaskBuilder::from_cache_path(if opts.no_cache {
                None
            } else {
                Some(std::path::Path::new(&cache_path))
            });
            let playbook = dotman::PlayBook::load_config(&opts.config, &task_builder)?;
            let verbose_lebel = if opts.verbose {
                VerboseLevel::ShowAllTask
            } else {
                VerboseLevel::Compact
            };
            let cache = if let Some(scenario) = opts.scenario {
                playbook.execute_graphicaly(false, Some(&scenario), &verbose_lebel)
            } else {
                playbook.execute_graphicaly(false, None, &verbose_lebel)
            }?;
            let mut f = fs::File::create(cache_path).map_err(|e| {
                dotman::Error::CannotLoadCache(format!("cannot write cache due to {:?}", e))
            })?;
            let cargo_cache = rmp_serde::from_read_ref(&cache.get("cargo").unwrap()).unwrap();
            let cache = Cache { cargo: cargo_cache };
            let writer = io::BufWriter::new(&mut f);
            serde_json::to_writer_pretty(writer, &cache).map_err(|e| {
                dotman::Error::CannotLoadCache(format!("cannot write cache due to {:?}", e))
            })?;
            Ok(())
        }
        Subcommand::DryRun(opts) => {
            let task_builder = TaskBuilder::from_cache_path(if opts.no_cache {
                None
            } else {
                Some(std::path::Path::new(&cache_path))
            });
            let playbook = dotman::PlayBook::load_config(&opts.config, &task_builder)?;
            let verbose_lebel = if opts.verbose {
                VerboseLevel::ShowAllTask
            } else {
                VerboseLevel::Compact
            };
            let _ = if let Some(scenario) = opts.scenario {
                playbook.execute_graphicaly(true, Some(&scenario), &verbose_lebel)
            } else {
                playbook.execute_graphicaly(true, None, &verbose_lebel)
            }?;
            Ok(())
        }
        Subcommand::Completion(completion_opts) => {
            let target = match completion_opts.shell.as_str() {
                "fish" => clap_generate::Shell::Fish,
                "zsh" => clap_generate::Shell::Zsh,
                "bash" => clap_generate::Shell::Bash,
                "powershell" => clap_generate::Shell::PowerShell,
                "elvish" => clap_generate::Shell::Elvish,
                _ => unreachable!(),
            };
            use clap::IntoApp;
            clap_generate::generate(target, &mut Opts::into_app(), "dotman", &mut io::stdout());
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
        Err(dotman::Error::CannotLoadCache(msg)) => {
            eprintln!(
                "{}[Error] {}cannot load cache due to {}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                msg
            );
        }
    }
}
