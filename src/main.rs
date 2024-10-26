use anyhow::{anyhow, Context, Result};
use dirs;
use std::fs::{rename, File, OpenOptions};
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

fn profiles_ini_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    path.push(".mozilla");
    path.push("firefox");
    path.push("profiles.ini");
    Ok(path)
}

#[derive(Eq, PartialEq)]
enum SectionState {
    BeforeFirstSection,
    InProfileSection,
    InOtherSection,
}

fn read_update_write<R: BufRead, W: Write>(reader: R, mut writer: W) -> Result<()> {
    let mut section_state = SectionState::BeforeFirstSection;
    let mut saw_profile = false;
    for line in reader.lines() {
        let line = line.context("Unable to read line from profiles.ini")?;
        if line.starts_with("[") {
            if section_state == SectionState::InProfileSection && !saw_profile {
                writeln!(writer, "Default=1")?;
                saw_profile = true;
            }
            if line.starts_with("[Profile") {
                section_state = SectionState::InProfileSection;
            } else {
                section_state = SectionState::InOtherSection;
            }
            writeln!(writer, "{}", line)?;
        } else {
            if section_state == SectionState::InProfileSection && line.eq("Default=1") {
                // skip line
            } else {
                writeln!(writer, "{}", line)?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let input_file = File::open(&profiles_ini_path()?)?;
    let reader = BufReader::new(input_file);
    read_update_write(reader, stdout())?;

    Ok(())
}
