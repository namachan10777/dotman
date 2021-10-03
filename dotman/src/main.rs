use kstring::KString;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use std::{env, fs, io, path, process};
use yaml_rust::{Yaml, YamlLoader};

use clap::{AppSettings, Clap};
use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq)]
enum TemplateValue {
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug)]
enum Task {
    Cp {
        src: String,
        dest: String,
        merge: bool,
        templates: Templates,
    },
}

impl Task {
    fn name(&self) -> String {
        match self {
            Task::Cp {
                src,
                dest,
                merge: _,
                templates: _,
            } => format!("cp {} => {}", src, dest),
        }
    }
}

#[derive(Debug)]
enum TargetMatcher {
    HostName(regex::Regex),
}

#[derive(Debug)]
struct Scenario {
    name: String,
    tasks: Vec<String>,
    matches: Vec<TargetMatcher>,
}

#[derive(Debug)]
struct PlayBook {
    taskgroups: HashMap<String, Vec<Task>>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone)]
struct TaskContext {
    base: PathBuf,
    dryrun: bool,
}

type Templates = HashMap<Vec<String>, liquid::Object>;
#[derive(Debug, Clone)]
struct CpContext {
    base: PathBuf,
    dryrun: bool,
    merge: bool,
    templates: Templates,
}

impl CpContext {
    fn extend(ctx: TaskContext, merge: bool, templates: Templates) -> Self {
        Self {
            merge,
            templates,
            base: ctx.base,
            dryrun: ctx.dryrun,
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    FailedToLoadPlaybook,
    TaskGroupNotFound(String),
    AnyScenarioDoesNotMatch,
    CannotResolveVar(String),
}

type TaskResult = Result<bool, TaskError>;
#[derive(Error, Debug)]
enum TaskError {
    #[error("wellknown {0}")]
    WellKnown(String),
    #[error("unknown {0}")]
    Unknown(anyhow::Error),
}

impl From<anyhow::Error> for TaskError {
    fn from(e: anyhow::Error) -> Self {
        TaskError::Unknown(e)
    }
}

#[derive(Debug, Clone)]
enum FileType {
    #[allow(dead_code)]
    Symlink(PathBuf),
    File(PathBuf),
    Other(PathBuf),
    Nothing(PathBuf),
    Dir(PathBuf),
}

fn resolve_desitination_path(path: &str) -> Result<PathBuf, Error> {
    Ok(Path::new(
        &path
            .split(path::MAIN_SEPARATOR)
            .map(|elem| {
                if let Some(var_name) = elem.strip_prefix('$') {
                    env::var(var_name).map_err(|_| Error::CannotResolveVar(elem.to_owned()))
                } else if elem.starts_with("\\$") {
                    Ok(elem[1..].to_owned())
                } else {
                    Ok(elem.to_owned())
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(&path::MAIN_SEPARATOR.to_string()),
    )
    .to_owned())
}

fn enlist_descendants(path: &Path) -> io::Result<Vec<PathBuf>> {
    if fs::metadata(path)?.is_dir() {
        let mut entries = fs::read_dir(path)?
            .into_iter()
            .map(|entry| enlist_descendants(&entry?.path()))
            .collect::<io::Result<Vec<_>>>()?
            .concat();
        entries.push(path.to_owned());
        Ok(entries)
    } else {
        Ok(vec![path.to_owned()])
    }
}

fn file_table(src: &Path, dest: &Path) -> io::Result<HashMap<PathBuf, (FileType, FileType)>> {
    let src_descendants = enlist_descendants(src)?;
    let dest_descendants = enlist_descendants(dest)?;
    let mut hash = HashMap::new();
    for src_descendant in src_descendants {
        let meta = fs::metadata(&src_descendant)?;
        let src_filetype = if meta.is_file() {
            FileType::File(src_descendant.to_owned())
        } else if meta.is_dir() {
            FileType::Dir(src_descendant.to_owned())
        } else {
            FileType::Other(src_descendant.to_owned())
        };
        hash.insert(
            src_descendant
                .strip_prefix(&Path::new(src))
                .unwrap()
                .to_owned(),
            (
                src_filetype,
                FileType::Nothing(dest.join(src_descendant.strip_prefix(src).unwrap())),
            ),
        );
    }
    for dest_descendant in dest_descendants {
        let meta = fs::metadata(&dest_descendant)?;
        let dest_filetype = if meta.is_file() {
            FileType::File(dest_descendant.to_owned())
        } else {
            FileType::Other(dest_descendant.to_owned())
        };
        hash.entry(
            dest_descendant
                .strip_prefix(&Path::new(dest))
                .unwrap()
                .to_owned(),
        )
        .and_modify(|pair| *pair = (pair.0.clone(), dest_filetype.clone()))
        .or_insert((
            FileType::Nothing(src.join(dest_descendant.strip_prefix(dest).unwrap())),
            dest_filetype,
        ));
    }
    Ok(hash)
}

fn match_template_target<'a>(
    templates: &'a Templates,
    src: &Path,
    target: &Path,
) -> Option<&'a liquid::Object> {
    for (target_patterns, var_set) in templates {
        let target = target.strip_prefix(src).unwrap_or(target);
        for pattern in target_patterns {
            if Path::new(pattern) == target {
                return Some(var_set);
            }
        }
    }
    None
}

#[derive(Error, Debug)]
enum SyncError {
    #[error("unhandled sync error {0}")]
    UnhandledIoError(io::Error),
    #[error("sync failed {0}")]
    Failed(String),
}

impl From<io::Error> for SyncError {
    fn from(e: io::Error) -> Self {
        SyncError::UnhandledIoError(e)
    }
}

enum SyncStatus {
    Changed,
    UnChanged,
    WellKnownError(String),
}

fn sync_file(ctx: &CpContext, src: &FileType, dest: &FileType) -> anyhow::Result<SyncStatus> {
    match (&src, &dest, ctx.merge) {
        (FileType::Nothing(_), FileType::Nothing(_), _) => Ok(SyncStatus::UnChanged),
        (FileType::Nothing(_), _, true) => Ok(SyncStatus::UnChanged),
        (FileType::Nothing(_), FileType::Dir(dest), false) => {
            // TODO: fix to unlink
            if !ctx.dryrun {
                fs::remove_dir(dest)?;
            }
            Ok(SyncStatus::Changed)
        }
        (FileType::Nothing(_), FileType::Symlink(dest), false) => {
            // TODO: fix to unlink
            if !ctx.dryrun {
                fs::remove_file(dest)?;
            }
            Ok(SyncStatus::Changed)
        }
        (FileType::Nothing(_), FileType::File(dest) | FileType::Other(dest), false) => {
            if !ctx.dryrun {
                fs::remove_file(dest)?;
            }
            Ok(SyncStatus::Changed)
        }
        (&FileType::File(src), &FileType::File(dest), _) => {
            let (src_buf, need_to_write) =
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, &src) {
                    match liquid::ParserBuilder::with_stdlib()
                        .build()
                        .unwrap()
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => (template.render(var_set)?.as_bytes().to_vec(), true),
                        Err(e) => {
                            return Ok(SyncStatus::WellKnownError(format!(
                                "cannot parse template {:?}",
                                src
                            )))
                        }
                    }
                } else {
                    (fs::read(src)?, false)
                };
            let dest_buf = fs::read(dest)?;
            if md5::compute(&src_buf) != md5::compute(dest_buf) {
                if !ctx.dryrun {
                    if need_to_write {
                        let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                        writer.write_all(&src_buf)?;
                        writer.flush()?;
                    } else {
                        fs::copy(src, dest)?;
                    }
                }
                Ok(SyncStatus::Changed)
            } else {
                Ok(SyncStatus::UnChanged)
            }
        }
        (&FileType::File(src), &FileType::Dir(dest), _) => {
            if !ctx.dryrun {
                fs::remove_dir(dest)?;
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, &src) {
                    let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                    match liquid::ParserBuilder::with_stdlib()
                        .build()
                        .unwrap()
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => {
                            writer.write_all(template.render(var_set)?.as_bytes())?;
                        }
                        Err(e) => {
                            return Ok(SyncStatus::WellKnownError(format!(
                                "cannot parse template {:?}",
                                src
                            )))
                        }
                    }
                } else {
                    fs::copy(src, dest)?;
                }
            }
            Ok(SyncStatus::Changed)
        }
        (&FileType::File(src), &FileType::Other(dest), _) => {
            if !ctx.dryrun {
                fs::remove_file(dest)?;
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, &src) {
                    let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                    match liquid::ParserBuilder::with_stdlib()
                        .build()
                        .unwrap()
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => {
                            writer.write_all(template.render(var_set)?.as_bytes())?;
                        }
                        Err(e) => {
                            return Ok(SyncStatus::WellKnownError(format!(
                                "cannot parse template {:?}",
                                src
                            )))
                        }
                    }
                } else {
                    fs::copy(src, dest)?;
                }
            }
            Ok(SyncStatus::Changed)
        }
        (&FileType::File(src), &FileType::Nothing(dest), _) => {
            if !ctx.dryrun {
                fs::create_dir_all(dest.parent().unwrap())?;
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, &src) {
                    let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                    match liquid::ParserBuilder::with_stdlib()
                        .build()
                        .unwrap()
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => {
                            writer.write_all(template.render(var_set)?.as_bytes())?;
                        }
                        Err(e) => {
                            return Ok(SyncStatus::WellKnownError(format!(
                                "cannot parse template {:?}",
                                src
                            )))
                        }
                    }
                } else {
                    fs::copy(src, dest)?;
                }
            }
            Ok(SyncStatus::Changed)
        }
        (&FileType::File(_), &FileType::Symlink(_), _) => Ok(SyncStatus::WellKnownError(
            "symlink is unsupported.".to_owned(),
        )),
        (FileType::Dir(_), _, _) => Ok(SyncStatus::UnChanged),
        (FileType::Other(_), _, _) => {
            Ok(SyncStatus::WellKnownError("unknown file type".to_owned()))
        }
        (FileType::Symlink(_), _, _) => Ok(SyncStatus::WellKnownError(
            "symlink is unsupported.".to_owned(),
        )),
    }
}

// TODO: handle error when src directory is not found.
fn execute_cp(ctx: &CpContext, src: &str, dest: &str) -> TaskResult {
    let src_base = ctx.base.join(Path::new(src));
    if let Ok(dest) = resolve_desitination_path(dest) {
        if let Ok(tbl) = file_table(&src_base, &dest) {
            let mut changed = false;
            for (src, dest) in tbl.values() {
                match sync_file(ctx, src, dest)? {
                    SyncStatus::Changed => {
                        changed = true;
                    }
                    SyncStatus::UnChanged => (),
                    SyncStatus::WellKnownError(msg) => {
                        return Err(TaskError::WellKnown(msg));
                    }
                }
            }
            return Ok(changed);
        }
    }
    Err(TaskError::WellKnown(format!(
        "cannot resolve disitination path {:?}",
        dest
    )))
}

fn execute(ctx: &TaskContext, task: &Task) -> TaskResult {
    match task {
        Task::Cp {
            src,
            dest,
            merge,
            templates,
        } => execute_cp(
            &CpContext::extend(ctx.clone(), *merge, templates.clone()),
            src,
            dest,
        ),
    }
}

fn parse_cp_templates(yaml: &Yaml) -> Result<(Vec<String>, liquid::Object), Error> {
    let hash = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    let target = match hash
        .get(&Yaml::String("target".to_owned()))
        .ok_or(Error::FailedToLoadPlaybook)?
    {
        Yaml::Array(targets) => targets
            .iter()
            .map(|target| {
                target
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or(Error::FailedToLoadPlaybook)
            })
            .collect::<Result<Vec<String>, Error>>(),
        Yaml::String(target) => Ok(vec![target.to_owned()]),
        _ => Err(Error::FailedToLoadPlaybook),
    }?;
    let mut context = liquid::Object::new();
    hash.get(&Yaml::String("vars".to_owned()))
        .ok_or(Error::FailedToLoadPlaybook)?
        .as_hash()
        .ok_or(Error::FailedToLoadPlaybook)?
        .into_iter()
        .map(|(name, val)| {
            let name =
                KString::from_string(name.as_str().ok_or(Error::FailedToLoadPlaybook)?.to_owned());
            match val {
                Yaml::String(str) => {
                    context.insert(name, liquid::model::Value::scalar(str.to_owned()))
                }
                Yaml::Integer(int) => context.insert(name, liquid::model::Value::scalar(*int)),
                Yaml::Real(float) => {
                    let f: f64 = float.parse().unwrap();
                    context.insert(name, liquid::model::Value::scalar(f))
                }
                _ => return Err(Error::FailedToLoadPlaybook),
            };
            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;
    Ok((target, context))
}

fn parse_task(yaml: &Yaml) -> Result<Task, Error> {
    let obj = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    if let Some(Yaml::String(key)) = obj.get(&Yaml::String("type".to_owned())) {
        match key.as_str() {
            "cp" => {
                let src = obj
                    .get(&Yaml::String("src".to_owned()))
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .as_str()
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .to_owned();
                let dest = obj
                    .get(&Yaml::String("dest".to_owned()))
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .as_str()
                    .ok_or(Error::FailedToLoadPlaybook)?
                    .to_owned();
                let merge = obj
                    .get(&Yaml::String("merge".to_owned()))
                    .map(|val| val.as_bool().ok_or(Error::FailedToLoadPlaybook))
                    .unwrap_or(Ok(true))?;
                let templates = obj
                    .get(&Yaml::String("templates".to_owned()))
                    .map(|templates| {
                        templates
                            .as_vec()
                            .ok_or(Error::FailedToLoadPlaybook)?
                            .iter()
                            .map(parse_cp_templates)
                            .collect::<Result<Templates, Error>>()
                    })
                    .unwrap_or_else(|| Ok(HashMap::new()))?;
                Ok(Task::Cp {
                    src,
                    dest,
                    merge,
                    templates,
                })
            }
            _ => Err(Error::FailedToLoadPlaybook),
        }
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn parse_taskgroups(yaml: &Yaml) -> Result<HashMap<String, Vec<Task>>, Error> {
    yaml.as_hash()
        .ok_or(Error::FailedToLoadPlaybook)?
        .iter()
        .map(|(name, tasks)| match (name, tasks) {
            (Yaml::String(name), Yaml::Array(tasks)) => Ok((
                name.to_owned(),
                tasks
                    .iter()
                    .map(parse_task)
                    .collect::<Result<Vec<Task>, Error>>()?,
            )),
            _ => Err(Error::FailedToLoadPlaybook),
        })
        .collect::<Result<HashMap<String, Vec<Task>>, Error>>()
}

fn parse_matcher(yaml: &Yaml) -> Result<TargetMatcher, Error> {
    let obj = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    if let Some((Yaml::String(target), val)) = obj.iter().next() {
        match target.as_str() {
            "hostname" => {
                let hostname_regex =
                    regex::Regex::new(val.as_str().ok_or(Error::FailedToLoadPlaybook)?)
                        .map_err(|_| Error::FailedToLoadPlaybook)?;
                Ok(TargetMatcher::HostName(hostname_regex))
            }
            _ => Err(Error::FailedToLoadPlaybook),
        }
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn parse_scenario(yaml: &Yaml) -> Result<Scenario, Error> {
    let obj = yaml.as_hash().ok_or(Error::FailedToLoadPlaybook)?;
    if let (Some(Yaml::String(name)), Some(Yaml::Array(matchers)), Some(Yaml::Array(tasks))) = (
        obj.get(&Yaml::String("name".to_owned())),
        obj.get(&Yaml::String("match".to_owned())),
        obj.get(&Yaml::String("tasks".to_owned())),
    ) {
        let matches = matchers
            .iter()
            .map(parse_matcher)
            .collect::<Result<Vec<TargetMatcher>, Error>>()?;
        let tasks = tasks
            .iter()
            .map(|taskname| {
                taskname
                    .as_str()
                    .map(|s| s.to_owned())
                    .ok_or(Error::FailedToLoadPlaybook)
            })
            .collect::<Result<Vec<String>, Error>>()?;
        Ok(Scenario {
            tasks,
            matches,
            name: name.to_owned(),
        })
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn load_config(config: &str) -> Result<PlayBook, Error> {
    let playbook_src =
        fs::read_to_string(Path::new(&config)).map_err(|_| Error::FailedToLoadPlaybook)?;
    let playbook_ast = YamlLoader::load_from_str(&playbook_src)
        .map_err(|_| Error::FailedToLoadPlaybook)?
        .get(0)
        .ok_or(Error::FailedToLoadPlaybook)?
        .as_hash()
        .ok_or(Error::FailedToLoadPlaybook)?
        .clone();
    if let (Some(taskgroups), Some(scenarios)) = (
        playbook_ast.get(&Yaml::String("taskgroups".to_owned())),
        playbook_ast.get(&Yaml::String("scenarios".to_owned())),
    ) {
        let taskgroups = parse_taskgroups(taskgroups)?;
        let scenarios = scenarios
            .as_vec()
            .ok_or(Error::FailedToLoadPlaybook)?
            .iter()
            .map(parse_scenario)
            .collect::<Result<Vec<Scenario>, Error>>()?;
        Ok(PlayBook {
            taskgroups,
            scenarios,
        })
    } else {
        Err(Error::FailedToLoadPlaybook)
    }
}

fn match_scenario(scenarios: &[Scenario]) -> Option<&Scenario> {
    for scenario in scenarios {
        if scenario.matches.iter().all(|matcher| match matcher {
            TargetMatcher::HostName(hostname_re) => hostname::get()
                .map(|hostname| hostname_re.is_match(&hostname.to_string_lossy()))
                .unwrap_or(false),
        }) {
            return Some(scenario);
        }
    }
    None
}

fn enlist_taskgroups<'a>(
    taskgroups: &'a HashMap<String, Vec<Task>>,
    taskgroup_names: &'a [String],
) -> Result<Vec<(&'a str, &'a [Task])>, Error> {
    taskgroup_names
        .iter()
        .map(|taskgroup_name| {
            taskgroups
                .get(taskgroup_name)
                .map(|task| (taskgroup_name.as_str(), task.as_slice()))
                .ok_or_else(|| Error::TaskGroupNotFound(taskgroup_name.to_owned()))
        })
        .collect::<Result<Vec<_>, Error>>()
}

type Stats = Vec<(String, Vec<(String, TaskResult)>)>;

fn execute_deploy(ctx: &TaskContext, playbook: &PlayBook) -> Result<Stats, Error> {
    let scenario = match_scenario(&playbook.scenarios).ok_or(Error::AnyScenarioDoesNotMatch)?;
    let taskgroups = enlist_taskgroups(&playbook.taskgroups, scenario.tasks.as_slice())?;
    Ok(taskgroups
        .iter()
        .map(|(name, tasks)| {
            (
                name.to_owned().to_owned(),
                tasks
                    .iter()
                    .map(|task| (task.name(), execute(ctx, task)))
                    .collect::<Vec<(String, TaskResult)>>(),
            )
        })
        .collect::<Stats>())
}

fn run(opts: Opts) -> Result<(), Error> {
    match opts.subcmd {
        Subcommand::Deploy(opts) => {
            let playbook = load_config(&opts.config).unwrap();
            let ctx = TaskContext {
                base: Path::new(&opts.config).parent().unwrap().to_owned(),
                dryrun: false,
            };
            let result = execute_deploy(&ctx, &playbook)?;
            println!("{:#?}", result);
            Ok(())
        }
        Subcommand::DryRun(opts) => {
            let playbook = load_config(&opts.config).unwrap();
            let ctx = TaskContext {
                base: Path::new(&opts.config).parent().unwrap().to_owned(),
                dryrun: true,
            };
            let result = execute_deploy(&ctx, &playbook)?;
            println!("{:#?}", result);
            Ok(())
        }
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match run(opts) {
        Ok(()) => (),
        Err(Error::AnyScenarioDoesNotMatch) => {
            eprintln!("any scenario does not match");
            process::exit(-1);
        }
        Err(Error::TaskGroupNotFound(taskgroup_name)) => {
            eprintln!("taskgroup \"{}\" does not found", taskgroup_name);
            process::exit(-1);
        }
        Err(Error::FailedToLoadPlaybook) => {
            eprintln!("failed to load playbook");
            process::exit(-1);
        }
        Err(Error::CannotResolveVar(var)) => {
            eprintln!("cannot resolve var ${}", var);
            process::exit(-1);
        }
    }
}
