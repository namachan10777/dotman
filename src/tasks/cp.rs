use kstring::KString;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;
use yaml_rust::{yaml::Hash, Yaml};

type Templates = HashMap<Vec<String>, liquid::Object>;

#[derive(Debug, Clone)]
enum FileType {
    #[allow(dead_code)]
    Symlink(PathBuf),
    File(PathBuf),
    Other(PathBuf),
    Nothing(PathBuf),
    Dir(PathBuf),
}

fn enlist_descendants(path: &Path) -> io::Result<Vec<PathBuf>> {
    if fs::metadata(path).is_err() {
        return Ok(Vec::new());
    }
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
fn execute_cp(ctx: &CpContext, src: &str, dest: &str) -> crate::TaskResult {
    let src_base = ctx.base.join(Path::new(src));
    if fs::metadata(&src_base).is_err() {
        return Err(crate::TaskError::WellKnown(format!(
            "src {:?} is not found",
            src_base
        )));
    }
    let dest = crate::util::resolve_desitination_path(dest).map_err(|e| {
        crate::TaskError::WellKnown(format!(
            "cannot resolve disitination path {:?} due to {:?}",
            dest, e
        ))
    })?;
    let tbl = file_table(&src_base, &dest).map_err(|e| {
        crate::TaskError::WellKnown(format!(
            "cannot resolve disitination path {:?} due to {:?}",
            dest, e
        ))
    })?;
    let mut changed = false;
    for (src, dest) in tbl.values() {
        match sync_file(ctx, src, dest)? {
            SyncStatus::Changed => {
                changed = true;
            }
            SyncStatus::UnChanged => (),
            SyncStatus::WellKnownError(msg) => {
                return Err(crate::TaskError::WellKnown(msg));
            }
        }
    }
    Ok(changed)
}

#[derive(Debug)]
pub struct CpTask {
    src: String,
    dest: String,
    merge: bool,
    templates: Templates,
}

#[derive(Debug, Clone)]
struct CpContext {
    base: PathBuf,
    dryrun: bool,
    merge: bool,
    templates: Templates,
}

impl CpContext {
    fn extend(ctx: &crate::TaskContext, merge: bool, templates: Templates) -> Self {
        let templates = templates
            .into_iter()
            .map(|(target, mut object)| {
                if !object.contains_key("_scenario") {
                    object.insert(
                        KString::from_static("_scenario"),
                        liquid::model::Value::scalar(ctx.scenario.clone()),
                    );
                }
                (target, object)
            })
            .collect::<HashMap<_, _>>();
        Self {
            merge,
            templates,
            base: ctx.base.clone(),
            dryrun: ctx.dryrun,
        }
    }
}

impl crate::Task for CpTask {
    fn name(&self) -> String {
        format!("cp {} => {}", self.src, self.dest)
    }

    fn execute(&self, ctx: &crate::TaskContext) -> crate::TaskResult {
        execute_cp(
            &CpContext::extend(ctx, self.merge, self.templates.clone()),
            &self.src,
            &self.dest,
        )
    }
}

fn parse_cp_templates(yaml: &Yaml) -> Result<(Vec<String>, liquid::Object), crate::Error> {
    let hash = yaml.as_hash().ok_or_else(|| {
        crate::Error::InvalidPlaybook("cp.templates must be hash".to_owned(), yaml.to_owned())
    })?;
    let target = match hash
        .get(&Yaml::String("target".to_owned()))
        .ok_or_else(|| {
            crate::Error::InvalidPlaybook(
                "cp.templates must have \"target\"".to_owned(),
                yaml.to_owned(),
            )
        })? {
        Yaml::Array(targets) => targets
            .iter()
            .map(|target| {
                target.as_str().map(|s| s.to_owned()).ok_or_else(|| {
                    crate::Error::InvalidPlaybook(
                        "cp.target must be string of array of string".to_owned(),
                        target.to_owned(),
                    )
                })
            })
            .collect::<Result<Vec<String>, crate::Error>>(),
        Yaml::String(target) => Ok(vec![target.to_owned()]),
        invalid => Err(crate::Error::InvalidPlaybook(
            "cp.target must be string of array of string".to_owned(),
            invalid.to_owned(),
        )),
    }?;
    let mut context = liquid::Object::new();
    hash.get(&Yaml::String("vars".to_owned()))
        .ok_or_else(|| {
            crate::Error::InvalidPlaybook("cp.template must have vars".to_owned(), yaml.to_owned())
        })?
        .as_hash()
        .ok_or_else(|| {
            crate::Error::InvalidPlaybook(
                "cp.templates.vars must be hash".to_owned(),
                yaml.to_owned(),
            )
        })?
        .into_iter()
        .map(|(name, val)| {
            let name = KString::from_string(
                name.as_str()
                    .ok_or_else(|| {
                        crate::Error::InvalidPlaybook(
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
                    return Err(crate::Error::InvalidPlaybook(
                        "children of cp.templates.vars must be string: <string|int|float>"
                            .to_owned(),
                        val.to_owned(),
                    ))
                }
            };
            Ok(())
        })
        .collect::<Result<Vec<()>, crate::Error>>()?;
    Ok((target, context))
}
pub fn parse(obj: &Hash) -> Result<Box<dyn crate::Task>, crate::Error> {
    let src = obj
        .get(&Yaml::String("src".to_owned()))
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cp must have \"src\"".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cp.src must be string".to_owned()))?
        .to_owned();
    let dest = obj
        .get(&Yaml::String("dest".to_owned()))
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cp must have \"dest\"".to_owned()))?
        .as_str()
        .ok_or_else(|| crate::Error::PlaybookLoadFailed("cp.dest must be string".to_owned()))?
        .to_owned();
    let merge = obj
        .get(&Yaml::String("merge".to_owned()))
        .map(|val| {
            val.as_bool().ok_or_else(|| {
                crate::Error::InvalidPlaybook("cp.merge must be boolean".to_owned(), val.to_owned())
            })
        })
        .unwrap_or(Ok(true))?;
    let templates = obj
        .get(&Yaml::String("templates".to_owned()))
        .map(|templates| {
            templates
                .as_vec()
                .ok_or_else(|| {
                    crate::Error::InvalidPlaybook(
                        "cp.templates must be array".to_owned(),
                        templates.to_owned(),
                    )
                })?
                .iter()
                .map(parse_cp_templates)
                .collect::<Result<Templates, crate::Error>>()
        })
        .unwrap_or_else(|| Ok(HashMap::new()))?;
    Ok(Box::new(CpTask {
        src,
        dest,
        merge,
        templates,
    }))
}
