//! Offline export of validated content and explicitly verified platform runtimes.
use super::{files, tree};
use crate::adventure::Adventure;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, BufRead, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    #[serde(rename = "windows-x64")]
    Windows,
    #[serde(rename = "linux-x64")]
    Linux,
}
impl Target {
    pub fn label(self) -> &'static str {
        match self {
            Self::Windows => "windows-x64",
            Self::Linux => "linux-x64",
        }
    }
    pub fn executable(self) -> &'static str {
        match self {
            Self::Windows => "lupine.exe",
            Self::Linux => "lupine",
        }
    }
    pub fn host() -> Option<Self> {
        if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            Some(Self::Windows)
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            Some(Self::Linux)
        } else {
            None
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeManifest {
    engine: String,
    version: String,
    export_format: u32,
    target: Target,
    sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn manifest(bytes: &[u8], target: Target) -> RuntimeManifest {
    RuntimeManifest {
        engine: "lupine".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        export_format: 1,
        target,
        sha256: digest(bytes),
    }
}
fn read_regular(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let m = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if files::is_link(&m) || !m.is_file() || m.len() > limit {
        return Err(format!(
            "{}: expected a regular file no larger than {limit} bytes",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.len() as u64 > limit {
        return Err(format!("{} grew past the size limit", path.display()));
    }
    Ok(bytes)
}
/// Emit beside a trusted native build as lupine.runtime.toml; never execute foreign files.
pub fn runtime_manifest() -> Result<String, String> {
    let target = Target::host().ok_or("Export supports Windows x64 and Linux x64 only")?;
    let bytes = read_regular(
        &std::env::current_exe().map_err(|e| e.to_string())?,
        256 * 1024 * 1024,
    )?;
    verify_header(&bytes, target)?;
    toml::to_string_pretty(&manifest(&bytes, target)).map_err(|e| e.to_string())
}
fn verify_header(bytes: &[u8], target: Target) -> Result<(), String> {
    let valid = match target {
        Target::Windows => {
            bytes.get(..2) == Some(b"MZ")
                && bytes.get(0x3c..0x40).is_some_and(|offset| {
                    let p = u32::from_le_bytes(offset.try_into().unwrap()) as usize;
                    bytes.get(p..p.saturating_add(6)) == Some(&b"PE\0\0\x64\x86"[..])
                        && bytes.get(p.saturating_add(24)..p.saturating_add(26))
                            == Some(&[0x0b, 0x02][..])
                })
        }
        Target::Linux => {
            bytes.get(..7) == Some(&b"\x7fELF\x02\x01\x01"[..])
                && bytes.get(7).is_some_and(|b| *b == 0 || *b == 3)
                && bytes.get(18..20) == Some(&[62, 0][..])
                && bytes
                    .get(16..18)
                    .is_some_and(|b| b == [2, 0] || b == [3, 0])
        }
    };
    if valid {
        Ok(())
    } else {
        Err(format!(
            "Binary is not a {} executable; obtain a native build for that platform",
            target.label()
        ))
    }
}
struct Runtime {
    target: Target,
    bytes: Vec<u8>,
    metadata: String,
}
fn runtime(path: &Path, target: Target) -> Result<Runtime, String> {
    let bytes = read_regular(path, 256 * 1024 * 1024)?;
    verify_header(&bytes, target)?;
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    let is_current =
        Target::host() == Some(target) && path.canonicalize().ok() == current.canonicalize().ok();
    let metadata = if is_current {
        toml::to_string_pretty(&manifest(&bytes, target)).map_err(|e| e.to_string())?
    } else {
        let sidecar = path.with_extension("runtime.toml");
        let raw=read_regular(&sidecar,16384).map_err(|e|format!("{e}\nProvide the matching lupine.runtime.toml from the same trusted build. On its native platform run lupine runtime-manifest to generate it. Export never downloads runtimes."))?;
        let raw = String::from_utf8(raw).map_err(|e| e.to_string())?;
        let info: RuntimeManifest =
            toml::from_str(&raw).map_err(|e| format!("{}: {e}", sidecar.display()))?;
        if info.engine != "lupine"
            || info.version != env!("CARGO_PKG_VERSION")
            || info.export_format != 1
            || info.target != target
            || info.sha256 != digest(&bytes)
        {
            return Err(format!("{}: runtime version, target, or SHA-256 mismatch. Use the executable and metadata from the same Lupine {} build.",path.display(),env!("CARGO_PKG_VERSION")));
        }
        raw
    };
    Ok(Runtime {
        target,
        bytes,
        metadata,
    })
}

pub struct ExportPlan {
    pub title: String,
    pub destination: PathBuf,
    content: BTreeMap<PathBuf, Vec<u8>>,
    runtimes: Vec<Runtime>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Zip,
    Folder,
}
impl ExportPlan {
    /// Preflight every target before creating anything. The plan owns immutable bytes.
    pub fn prepare(
        source: &Path,
        destination: &Path,
        binaries: &[(Target, PathBuf)],
    ) -> Result<Self, String> {
        let source = tree::root(source)?;
        if source.join(".lupine-room-edit").exists() {
            return Err("Finish/review the adventure's pending recovery before export.".into());
        }
        let report = tree::inspect(&source);
        if !report.problems.is_empty() {
            return Err(report.problems.join("\n"));
        }
        let adventure = Adventure::load(&source)?;
        files::validate_destination(destination)?;
        let parent = destination
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = parent.canonicalize().map_err(|e| e.to_string())?;
        if parent.starts_with(source.canonicalize().map_err(|e| e.to_string())?) {
            return Err(
                "Export must be outside the source adventure. Choose a sibling folder.".into(),
            );
        }
        let destination = std::path::absolute(destination).map_err(|e| e.to_string())?;
        if binaries.is_empty() {
            return Err("Select at least one platform".into());
        }
        let mut runtimes = Vec::new();
        for (target, path) in binaries {
            if runtimes.iter().any(|r: &Runtime| r.target == *target) {
                return Err("Duplicate export platform".into());
            }
            runtimes.push(runtime(path, *target)?);
        }
        let mut content = BTreeMap::new();
        for name in [
            "events.toml",
            "dialogue.toml",
            "systems.toml",
            "actions.toml",
            "rules.toml",
            "game.toml",
            "interactions.toml",
            "transaction_interactions.toml",
        ] {
            if source.join(name).exists() {
                content.insert(
                    PathBuf::from(name),
                    read_regular(&source.join(name), 8 * 1024 * 1024)?,
                );
            }
        }
        for name in [
            "rooms", "items", "spells", "weapons", "enemies", "shops", "config", "art",
        ] {
            if source.join(name).exists() {
                collect(&source, Path::new(name), &mut content)?;
            }
        }
        Ok(Self {
            title: adventure.game.title,
            destination,
            content,
            runtimes,
        })
    }
    pub fn preview(&self, output: &mut impl Write) -> io::Result<()> {
        writeln!(output,"Adventure: {}\nExport folder: {}\nPlatforms: {}\nAdventure files (no saves or development files):",self.title,self.destination.display(),self.runtimes.iter().map(|r|r.target.label()).collect::<Vec<_>>().join(", "))?;
        for (path, bytes) in &self.content {
            writeln!(output, "  {} ({} bytes)", path.display(), bytes.len())?;
        }
        writeln!(output,"Each platform folder includes the executable, adventure/, START_HERE.txt, a launcher and engine license. Source files stay unchanged.")
    }
    pub fn publish(self) -> Result<PathBuf, String> {
        self.publish_format(OutputFormat::Folder)
    }
    pub fn publish_format(mut self, format: OutputFormat) -> Result<PathBuf, String> {
        files::validate_destination(&self.destination)?;
        let destination = self.destination.clone();
        let stage = super::export_workspace::reserve(
            destination.parent().ok_or("Missing destination parent")?,
        )?;
        self.destination = stage.join("packages");
        let result = (|| {
            self.write_packages()?;
            let output = if format == OutputFormat::Zip {
                let out = stage.join("archives");
                fs::create_dir(&out).map_err(|e| e.to_string())?;
                let slug = self
                    .title
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                    .take(80)
                    .collect::<String>();
                for runtime in &self.runtimes {
                    let name = format!(
                        "{}-{}.zip",
                        if slug.trim_matches('_').is_empty() {
                            "Adventure"
                        } else {
                            &slug
                        },
                        runtime.target.label()
                    );
                    let part = out.join(format!("{name}.part"));
                    super::archive::create(&self.destination.join(runtime.target.label()), &part)?;
                    fs::hard_link(&part, out.join(name)).map_err(|e| e.to_string())?;
                    fs::remove_file(part).map_err(|e| e.to_string())?;
                }
                out
            } else {
                self.destination.clone()
            };
            super::export_workspace::record(&stage)?;
            let inventory = super::archive::inventory(&output)?;
            // Reserve the final directory exclusively; never replace an existing export.
            files::validate_destination(&destination)?;
            fs::create_dir(&destination).map_err(|e| e.to_string())?;
            write(&destination.join("EXPORT_INCOMPLETE.txt"),b"Export publication incomplete. Do not distribute. Retry into a new destination. Preserve unknown or modified files; no source adventure files changed.\n")?;
            for (_, relative, dir) in &inventory {
                if *dir {
                    fs::create_dir(destination.join(relative)).map_err(|e| e.to_string())?;
                } else {
                    let source = output.join(relative);
                    let expected = super::archive::file_hash(&source)?;
                    fs::hard_link(&source, destination.join(relative))
                        .map_err(|e| e.to_string())?;
                    if super::archive::file_hash(&destination.join(relative))? != expected {
                        return Err("Published export digest mismatch".into());
                    }
                }
            }
            fs::remove_file(destination.join("EXPORT_INCOMPLETE.txt"))
                .map_err(|e| e.to_string())?;
            super::export_workspace::cleanup(&stage).map_err(|e| {
                format!(
                    "Export is complete at {}, but staging cleanup needs attention: {e}",
                    destination.display()
                )
            })?;
            Ok(destination.clone())
        })();
        result.map_err(|e:String|format!("{e}\nExport workspace: {}. Use Export > C for verified cleanup when a complete record exists; otherwise preserve it for manual review. No source files changed.",stage.display()))
    }
    fn write_packages(&self) -> Result<(), String> {
        files::validate_destination(&self.destination)?;
        fs::create_dir(&self.destination).map_err(|e| e.to_string())?;
        let result = (|| {
            write(&self.destination.join("EXPORT_INCOMPLETE.txt"),b"Export incomplete. Do not distribute or launch. Retry into a new folder. The source adventure is unchanged.\n")?;
            for runtime in &self.runtimes {
                let package = self.destination.join(runtime.target.label());
                fs::create_dir(&package).map_err(|e| e.to_string())?;
                let adventure = package.join("adventure");
                fs::create_dir(&adventure).map_err(|e| e.to_string())?;
                for name in [
                    "rooms", "items", "spells", "weapons", "enemies", "shops", "config", "art",
                ] {
                    fs::create_dir(adventure.join(name)).map_err(|e| e.to_string())?;
                }
                for (path, bytes) in &self.content {
                    let destination = adventure.join(path);
                    fs::create_dir_all(destination.parent().unwrap()).map_err(|e| e.to_string())?;
                    write(&destination, bytes)?;
                }
                Adventure::load(&adventure)?;
                write(&package.join(runtime.target.executable()), &runtime.bytes)?;
                write(
                    &package.join("lupine.runtime.toml"),
                    runtime.metadata.as_bytes(),
                )?;
                write(
                    &package.join("LUPINE_LICENSE.txt"),
                    include_bytes!("../../LICENSE"),
                )?;
                write(
                    &package.join("THIRD_PARTY_NOTICES.txt"),
                    include_bytes!("../../THIRD_PARTY_NOTICES.txt"),
                )?;
                let instructions=format!("{}\nLupine {} - {}\n\nWindows: double-click Play.cmd, or open a terminal here and run .\\lupine.exe play adventure\nLinux: open a terminal here and run sh Play.sh\nLinux requires a compatible x86_64 Linux system and the runtime's system libraries (official builds use glibc).\nNo Rust, Cargo, account or internet is needed.\n\nThe executable alone opens Lupine's startup menu; choose Play and the included adventure.\nUse help in gameplay. Save only with save <name>; load <name> restores manually.\nPlayer saves live in your user-data directory, outside this package. Nothing autosaves.\n\nShare this complete platform folder, optionally zipped with your normal archive tool.\nDo not mix Windows and Linux executables. Include your own story license/credits as appropriate.\n",self.title,env!("CARGO_PKG_VERSION"),runtime.target.label());
                write(&package.join("START_HERE.txt"), instructions.as_bytes())?;
            }
            // Publish launchers only after every package has validated.
            for runtime in &self.runtimes {
                let package = self.destination.join(runtime.target.label());
                match runtime.target {
                    Target::Windows => write(&package.join("Play.cmd"),b"@echo off\r\nsetlocal DisableDelayedExpansion\r\ncd /d \"%~dp0\"\r\nlupine.exe play adventure\r\npause\r\n")?,
                    Target::Linux => {
                        write(&package.join("Play.sh"),b"#!/bin/sh\nset -eu\ncd -- \"$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\"\nchmod u+x ./lupine\nexec ./lupine play adventure\n")?;
                        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; fs::set_permissions(package.join("lupine"),fs::Permissions::from_mode(0o755)).map_err(|e|e.to_string())?; fs::set_permissions(package.join("Play.sh"),fs::Permissions::from_mode(0o755)).map_err(|e|e.to_string())?; }
                    }
                }
            }
            fs::remove_file(self.destination.join("EXPORT_INCOMPLETE.txt"))
                .map_err(|e| e.to_string())?;
            Ok(())
        })();
        result.map_err(|error:String|format!("{error}\nExport incomplete at {}. Do not distribute it. No source files changed; retry into a new folder.",self.destination.display()))?;
        Ok(())
    }
}
fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn collect(
    root: &Path,
    relative: &Path,
    content: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<(), String> {
    if relative.components().count() > 9 || content.len() > 2048 {
        return Err("Export exceeds safe traversal limits".into());
    }
    for entry in fs::read_dir(root.join(relative)).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let rel = path.strip_prefix(root).map_err(|e| e.to_string())?;
        let meta = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if files::is_link(&meta) {
            return Err(format!("{}: export refuses linked content", path.display()));
        }
        if meta.is_dir() {
            if matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some(".git" | "target" | ".lupine-room-edit")
            ) {
                continue;
            }
            if relative.starts_with("art") {
                collect(root, rel, content)?;
            }
            continue;
        }
        let ext = path.extension().and_then(|s| s.to_str());
        let allowed = if relative.starts_with("art") {
            ext == Some("txt")
        } else if relative == Path::new("config") {
            matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("character.fig" | "terminology.fig" | "combat.fig")
            )
        } else {
            ext == Some("toml")
        };
        if allowed {
            let bytes = read_regular(&path, 8 * 1024 * 1024)?;
            if content.values().map(Vec::len).sum::<usize>() + bytes.len() > 64 * 1024 * 1024 {
                return Err("Adventure exceeds 64 MiB export limit".into());
            }
            content.insert(rel.to_owned(), bytes);
        }
    }
    Ok(())
}

fn prompt(
    input: &mut impl BufRead,
    output: &mut impl Write,
    label: &str,
) -> io::Result<Option<String>> {
    writeln!(output, "{label}\n(back/cancel returns)")?;
    write!(output, "> ")?;
    output.flush()?;
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    let line = line.trim();
    if matches!(line.to_ascii_lowercase().as_str(), "back" | "cancel" | "0") {
        Ok(None)
    } else {
        Ok(Some(line.into()))
    }
}
pub fn run(source: &Path, input: &mut impl BufRead, output: &mut impl Write) -> io::Result<()> {
    super::ui::heading(output, "EXPORT YOUR ADVENTURE")?;
    writeln!(output,"Export creates a new distribution folder containing ZIPs or platform folders.\n  1. Windows x64\n  2. Linux x64\n  3. Both platforms\n  C. Clean a recorded export workspace\n  0. Back")?;
    let targets = loop {
        let Some(choice) = prompt(input, output, "Choose target platform:")? else {
            return Ok(());
        };
        match choice.as_str() {
            "c" | "C" => {
                let Some(path) = prompt(
                    input,
                    output,
                    "Exact .lupine-export-* workspace path (not the final export):",
                )?
                else {
                    return Ok(());
                };
                writeln!(output,"Cleanup refuses unknown or modified content. Source adventures and final packages are not cleanup targets.")?;
                if prompt(
                    input,
                    output,
                    "Confirm verified workspace cleanup? Type yes:",
                )?
                .is_some_and(|s| s.eq_ignore_ascii_case("yes"))
                {
                    match super::export_workspace::cleanup(Path::new(&path)) {
                        Ok(()) => writeln!(output, "Verified workspace removed.")?,
                        Err(e) => writeln!(output, "Cleanup refused: {e}")?,
                    }
                }
                return Ok(());
            }
            "1" => break vec![Target::Windows],
            "2" => break vec![Target::Linux],
            "3" => break vec![Target::Windows, Target::Linux],
            _ => writeln!(output, "Choose 1, 2, 3 or 0.")?,
        }
    };
    let format = loop {
        let Some(choice)=prompt(input,output,"Output format:\n  1. Ready-to-share ZIP (stored, no compression)\n  2. Uncompressed Folder\n  0. Cancel")? else{return Ok(());};
        match choice.as_str() {
            "1" => break OutputFormat::Zip,
            "2" => break OutputFormat::Folder,
            _ => writeln!(output, "Choose 1 or 2.")?,
        }
    };
    let mut binaries = Vec::new();
    for target in targets {
        let host = Target::host() == Some(target);
        let message = format!(
            "{} runtime executable path{}:",
            target.label(),
            if host {
                " (Enter uses this running Lupine)"
            } else {
                " (needs matching lupine.runtime.toml from the same trusted build)"
            }
        );
        let Some(path) = prompt(input, output, &message)? else {
            return Ok(());
        };
        let path = if path.is_empty() && host {
            std::env::current_exe()?
        } else {
            PathBuf::from(path)
        };
        binaries.push((target, path));
    }
    let Some(destination) = prompt(
        input,
        output,
        "New export folder outside the adventure (parent must exist; no surrounding quotes):",
    )?
    else {
        return Ok(());
    };
    let plan = match ExportPlan::prepare(source, Path::new(&destination), &binaries) {
        Ok(p) => p,
        Err(e) => {
            writeln!(output, "Cannot export:\n{e}")?;
            return Ok(());
        }
    };
    plan.preview(output)?;
    writeln!(output,"Output format: {format:?}. Both platforms produce separate packages. ZIP entries are bounded to 256 MiB each and 512 MiB per archive.")?;
    let Some(answer) = prompt(
        input,
        output,
        "Create this export? Type yes to confirm; Enter declines:",
    )?
    else {
        return Ok(());
    };
    if !answer.eq_ignore_ascii_case("yes") {
        writeln!(output, "Export canceled. Nothing written.")?;
        return Ok(());
    }
    match plan.publish_format(format) {
        Ok(path) => {
            super::ui::heading(output, "EXPORT COMPLETE!")?;
            writeln!(output,"Package folder: {}\nShare the ZIP for the player's platform, or its complete platform folder.\nExtract ZIPs before launching. Players follow START_HERE.txt; no Rust installation is needed.",path.display())?;
        }
        Err(e) => writeln!(output, "Export failed:\n{e}")?,
    }
    Ok(())
}
