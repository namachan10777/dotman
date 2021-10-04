use kstring::KString;
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use std::{env, fs, io, path};
use yaml_rust::{Yaml, YamlLoader};

use thiserror::Error;

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
pub struct PlayBook {
    taskgroups: HashMap<String, Vec<Task>>,
    scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub base: PathBuf,
    pub dryrun: bool,
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
pub enum Error {
    PlaybookLoadFailed(String),
    InvalidPlaybook(String, Yaml),
    TaskGroupNotFound(String),
    AnyScenarioDoesNotMatch,
    CannotResolveVar(String),
}

type TaskResult = Result<bool, TaskError>;
#[derive(Error, Debug)]
pub enum TaskError {
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

fn file_table(src: &Path, dest: &Path) -> anyhow::Result<HashMap<PathBuf, (FileType, FileType)>> {
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
            src_descendant.strip_prefix(&Path::new(src))?.to_owned(),
            (
                src_filetype,
                FileType::Nothing(dest.join(src_descendant.strip_prefix(src)?)),
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
        hash.entry(dest_descendant.strip_prefix(&Path::new(dest))?.to_owned())
            .and_modify(|pair| *pair = (pair.0.clone(), dest_filetype.clone()))
            .or_insert((
                FileType::Nothing(src.join(dest_descendant.strip_prefix(dest)?)),
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

enum SyncStatus {
    Changed,
    UnChanged,
    WellKnownError(String),
}

#[derive(Error, Debug)]
struct SyncError {
    msg: String,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SyncError({})", self.msg))
    }
}

impl SyncError {
    fn new(msg: String) -> Self {
        Self { msg }
    }
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
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, src) {
                    match liquid::ParserBuilder::with_stdlib()
                        .build()?
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => (template.render(var_set)?.as_bytes().to_vec(), true),
                        Err(_) => {
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
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, src) {
                    let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                    match liquid::ParserBuilder::with_stdlib()
                        .build()?
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => {
                            writer.write_all(template.render(var_set)?.as_bytes())?;
                        }
                        Err(_) => {
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
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, src) {
                    let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                    match liquid::ParserBuilder::with_stdlib()
                        .build()?
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => {
                            writer.write_all(template.render(var_set)?.as_bytes())?;
                        }
                        Err(_) => {
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
                let dest_parent = dest
                    .parent()
                    .ok_or_else(|| SyncError::new(format!("cannot take parent of {:?}", dest)))?;
                fs::create_dir_all(dest_parent)?;
                if let Some(var_set) = match_template_target(&ctx.templates, &ctx.base, src) {
                    let mut writer = io::BufWriter::new(fs::File::create(dest)?);
                    match liquid::ParserBuilder::with_stdlib()
                        .build()?
                        .parse(&fs::read_to_string(src)?)
                    {
                        Ok(template) => {
                            writer.write_all(template.render(var_set)?.as_bytes())?;
                        }
                        Err(_) => {
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
    let hash = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("cp.templates must be hash".to_owned(), yaml.to_owned())
    })?;
    let target = match hash
        .get(&Yaml::String("target".to_owned()))
        .ok_or_else(|| {
            Error::InvalidPlaybook(
                "cp.templates must have \"target\"".to_owned(),
                yaml.to_owned(),
            )
        })? {
        Yaml::Array(targets) => targets
            .iter()
            .map(|target| {
                target.as_str().map(|s| s.to_owned()).ok_or_else(|| {
                    Error::InvalidPlaybook(
                        "cp.target must be string of array of string".to_owned(),
                        target.to_owned(),
                    )
                })
            })
            .collect::<Result<Vec<String>, Error>>(),
        Yaml::String(target) => Ok(vec![target.to_owned()]),
        invalid => Err(Error::InvalidPlaybook(
            "cp.target must be string of array of string".to_owned(),
            invalid.to_owned(),
        )),
    }?;
    let mut context = liquid::Object::new();
    hash.get(&Yaml::String("vars".to_owned()))
        .ok_or_else(|| {
            Error::InvalidPlaybook("cp.template must have vars".to_owned(), yaml.to_owned())
        })?
        .as_hash()
        .ok_or_else(|| {
            Error::InvalidPlaybook("cp.templates.vars must be hash".to_owned(), yaml.to_owned())
        })?
        .into_iter()
        .map(|(name, val)| {
            let name = KString::from_string(
                name.as_str()
                    .ok_or_else(|| {
                        Error::InvalidPlaybook(
                            "children of cp.templates.vars must be string: <string|int|float>"
                                .to_owned(),
                            name.to_owned(),
                        )
                    })?
                    .to_owned(),
            );
            match val {
                Yaml::String(str) => {
                    context.insert(name, liquid::model::Value::scalar(str.to_owned()))
                }
                Yaml::Integer(int) => context.insert(name, liquid::model::Value::scalar(*int)),
                Yaml::Real(float) => {
                    let f: f64 = float.parse().expect("already parse as real by yaml parser");
                    context.insert(name, liquid::model::Value::scalar(f))
                }
                _ => {
                    return Err(Error::InvalidPlaybook(
                        "children of cp.templates.vars must be string: <string|int|float>"
                            .to_owned(),
                        val.to_owned(),
                    ))
                }
            };
            Ok(())
        })
        .collect::<Result<Vec<()>, Error>>()?;
    Ok((target, context))
}

fn parse_task(yaml: &Yaml) -> Result<Task, Error> {
    let obj = yaml
        .as_hash()
        .ok_or_else(|| Error::InvalidPlaybook("task must be hash".to_owned(), yaml.to_owned()))?;
    if let Some(Yaml::String(key)) = obj.get(&Yaml::String("type".to_owned())) {
        match key.as_str() {
            "cp" => {
                let src = obj
                    .get(&Yaml::String("src".to_owned()))
                    .ok_or_else(|| {
                        Error::InvalidPlaybook("cp must have \"src\"".to_owned(), yaml.to_owned())
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        Error::InvalidPlaybook("cp.src must be string".to_owned(), yaml.to_owned())
                    })?
                    .to_owned();
                let dest = obj
                    .get(&Yaml::String("dest".to_owned()))
                    .ok_or_else(|| {
                        Error::InvalidPlaybook("cp must have \"dest\"".to_owned(), yaml.to_owned())
                    })?
                    .as_str()
                    .ok_or_else(|| {
                        Error::InvalidPlaybook("cp.dest must be string".to_owned(), yaml.to_owned())
                    })?
                    .to_owned();
                let merge = obj
                    .get(&Yaml::String("merge".to_owned()))
                    .map(|val| {
                        val.as_bool().ok_or_else(|| {
                            Error::InvalidPlaybook(
                                "cp.merge must be boolean".to_owned(),
                                val.to_owned(),
                            )
                        })
                    })
                    .unwrap_or(Ok(true))?;
                let templates = obj
                    .get(&Yaml::String("templates".to_owned()))
                    .map(|templates| {
                        templates
                            .as_vec()
                            .ok_or_else(|| {
                                Error::InvalidPlaybook(
                                    "cp.templates must be array".to_owned(),
                                    templates.to_owned(),
                                )
                            })?
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
            taskname => Err(Error::InvalidPlaybook(
                format!("unsupported task \"{}\"", taskname),
                yaml.to_owned(),
            )),
        }
    } else {
        Err(Error::InvalidPlaybook(
            "task must have \"type\" property".to_owned(),
            yaml.to_owned(),
        ))
    }
}

fn parse_taskgroups(yaml: &Yaml) -> Result<HashMap<String, Vec<Task>>, Error> {
    yaml.as_hash()
        .ok_or_else(|| {
            Error::InvalidPlaybook("taskgroups must be hash".to_owned(), yaml.to_owned())
        })?
        .iter()
        .map(|(name, tasks)| match (name, tasks) {
            (Yaml::String(name), Yaml::Array(tasks)) => Ok((
                name.to_owned(),
                tasks
                    .iter()
                    .map(parse_task)
                    .collect::<Result<Vec<Task>, Error>>()?,
            )),
            _ => Err(Error::InvalidPlaybook(
                "children of taskgropus must be [string]: <task>[]".to_owned(),
                yaml.to_owned(),
            )),
        })
        .collect::<Result<HashMap<String, Vec<Task>>, Error>>()
}

fn parse_matcher(yaml: &Yaml) -> Result<TargetMatcher, Error> {
    let obj = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("matcher must be hash".to_owned(), yaml.to_owned())
    })?;
    if let Some((Yaml::String(target), val)) = obj.iter().next() {
        match target.as_str() {
            "hostname" => {
                let hostname_re_src = val.as_str().ok_or_else(|| {
                    Error::InvalidPlaybook(
                        "matcher.hostname must be string".to_owned(),
                        val.to_owned(),
                    )
                })?;
                let hostname_regex = regex::Regex::new(hostname_re_src).map_err(|e| {
                    Error::InvalidPlaybook(
                        format!(
                            "cannot compile matcher.hostname {} due to {:?}",
                            hostname_re_src, e
                        ),
                        val.to_owned(),
                    )
                })?;
                Ok(TargetMatcher::HostName(hostname_regex))
            }
            matcher_name => Err(Error::InvalidPlaybook(
                format!("unsupported matcher \"{}\"", matcher_name),
                yaml.to_owned(),
            )),
        }
    } else {
        Err(Error::InvalidPlaybook(
            "matcher must be [string]: <matcher>".to_owned(),
            yaml.to_owned(),
        ))
    }
}

fn parse_scenario(yaml: &Yaml) -> Result<Scenario, Error> {
    let obj = yaml.as_hash().ok_or_else(|| {
        Error::InvalidPlaybook("scenario mast be hash".to_owned(), yaml.to_owned())
    })?;
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
                taskname.as_str().map(|s| s.to_owned()).ok_or_else(|| {
                    Error::InvalidPlaybook(
                        "scenario.tasks must be array of string".to_owned(),
                        yaml.to_owned(),
                    )
                })
            })
            .collect::<Result<Vec<String>, Error>>()?;
        Ok(Scenario {
            tasks,
            matches,
            name: name.to_owned(),
        })
    } else {
        Err(Error::InvalidPlaybook(
            "scenario.name must be string, scenario.match and scenario.tasks must be array"
                .to_owned(),
            yaml.to_owned(),
        ))
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

pub type Stats = Vec<(String, Vec<(String, TaskResult)>)>;

impl PlayBook {
    pub fn load_config(config: &str) -> Result<Self, Error> {
        let playbook_src = fs::read_to_string(Path::new(&config)).map_err(|e| {
            Error::PlaybookLoadFailed(format!("cannot read playbook {} due to {:?}", config, e))
        })?;
        let playbook_ast = YamlLoader::load_from_str(&playbook_src)
            .map_err(|_| {
                Error::PlaybookLoadFailed(format!("playbook {} has invalid syntax", config))
            })?
            .get(0)
            .ok_or_else(|| Error::PlaybookLoadFailed(format!("playbook {} is empty", config)))?
            .as_hash()
            .ok_or_else(|| Error::PlaybookLoadFailed(format!("playbook {} is not a hash", config)))?
            .clone();
        let taskgroups = playbook_ast
            .get(&Yaml::String("taskgroups".to_owned()))
            .ok_or_else(|| Error::PlaybookLoadFailed("taskgroups is not found".to_owned()))?;
        let scenarios = playbook_ast
            .get(&Yaml::String("scenarios".to_owned()))
            .ok_or_else(|| Error::PlaybookLoadFailed("scenarios is not found".to_owned()))?;
        let taskgroups = parse_taskgroups(taskgroups)?;
        let scenarios = scenarios
            .as_vec()
            .ok_or_else(|| {
                Error::InvalidPlaybook("scenario must be array".to_owned(), scenarios.to_owned())
            })?
            .iter()
            .map(parse_scenario)
            .collect::<Result<Vec<Scenario>, Error>>()?;
        Ok(PlayBook {
            taskgroups,
            scenarios,
        })
    }
    pub fn execute_deploy(&self, ctx: &TaskContext) -> Result<Stats, Error> {
        let scenario = match_scenario(&self.scenarios).ok_or(Error::AnyScenarioDoesNotMatch)?;
        let taskgroups = enlist_taskgroups(&self.taskgroups, scenario.tasks.as_slice())?;
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
}
