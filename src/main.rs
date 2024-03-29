use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use dotman::VerboseLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::{fs, io, process};
use termion::color;
#[derive(Parser)]
#[command(name = "dotman")]
struct Opts {
    #[clap(subcommand)]
    subcmd: Subcommand,
}

#[derive(Parser)]
enum Subcommand {
    #[clap(override_help = "deploy dotfiles")]
    Deploy(DeployOpts),
    #[clap(override_help = "dry run")]
    DryRun(DryRunOpts),
    #[clap(override_help = "generate shell completion")]
    Completion(CompletionOpts),
}

#[derive(Parser)]
struct CompletionOpts {
    #[clap(short, long)]
    shell: Shell,
}

#[derive(Parser)]
struct DeployOpts {
    #[clap(index = 1, help = "specify configuration file e.g. \"dotfiles.yaml\"")]
    config: String,
    #[clap(long = "no-cache", help = "deploy without cache")]
    no_cache: bool,
    #[clap(short, long, help = "specify scenario with no auto scenario detection")]
    #[clap(short = 's', long = "scenario")]
    scenario: Option<String>,
    #[clap(short = 'V', long)]
    verbose: bool,
}

#[derive(Parser)]
struct DryRunOpts {
    #[clap(index = 1, help = "specify configuration file e.g. \"dotfiles.yaml\"")]
    config: String,
    #[clap(long = "no-cache", help = "dry-run without cache")]
    no_cache: bool,
    #[clap(short, long, help = "specify scenario with no auto scenario detection")]
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
    serialize_ids: Vec<&'static str>,
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
            #[cfg(target_os = "macos")]
            "brew" => Some(dotman::tasks::brew::parse(hash)),
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

    fn serialize_ids(&self) -> &[&str] {
        self.serialize_ids.as_slice()
    }
}

impl TaskBuilder {
    fn from_cache_path<P: AsRef<Path>>(path: Option<P>) -> Self {
        let mut ids = vec!["cp", "env", "sh", "cargo", "link"];
        #[cfg(target_os = "macos")]
        let serialize_ids = vec!["cargo", "brew"];
        #[cfg(not(target_os = "macos"))]
        let serialize_ids = vec!["cargo"];
        #[cfg(feature = "network")]
        ids.push("wget");
        #[cfg(target_os = "macos")]
        ids.push("brew");
        if let Some(path) = path {
            if let Ok(f) = fs::File::open(path) {
                if let Ok(cache) = serde_json::from_reader(f) {
                    return Self {
                        ids,
                        cache,
                        serialize_ids,
                    };
                }
            }
        };
        Self {
            cache: Cache { cargo: None },
            ids,
            serialize_ids,
        }
    }
}

async fn run(opts: Opts) -> Result<(), dotman::Error> {
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
                playbook
                    .execute_graphicaly(false, Some(&scenario), &verbose_lebel)
                    .await
            } else {
                playbook
                    .execute_graphicaly(false, None, &verbose_lebel)
                    .await
            }?;
            let mut f = fs::File::create(cache_path).map_err(|e| {
                dotman::Error::CannotLoadCache(format!("cannot write cache due to {:?}", e))
            })?;
            let cargo_cache =
                rmp_serde::from_read(io::Cursor::new(&cache.get("cargo").unwrap())).unwrap();
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
                playbook
                    .execute_graphicaly(true, Some(&scenario), &verbose_lebel)
                    .await
            } else {
                playbook
                    .execute_graphicaly(true, None, &verbose_lebel)
                    .await
            }?;
            Ok(())
        }
        Subcommand::Completion(completion_opts) => {
            let generator = completion_opts.shell;
            let mut cmd = Opts::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(generator, &mut cmd, name, &mut io::stdout());
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    match run(opts).await {
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
