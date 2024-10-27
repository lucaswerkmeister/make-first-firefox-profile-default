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
    if section_state == SectionState::InProfileSection && !saw_profile {
        writeln!(writer, "Default=1")?;
        saw_profile = true;
    }
    Ok(())
}

fn main() -> Result<()> {
    let input_file = File::open(&profiles_ini_path()?)?;
    let reader = BufReader::new(input_file);
    read_update_write(reader, stdout())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn one_profile() {
        let input = r#"[Install0123456789ABCDEF]
Default=abcdefgh.default
Locked=1

[Profile0]
Name=default
IsRelative=1
Path=abcdefgh.default
Default=1
"#;
        let mut output = Vec::new();
        read_update_write(BufReader::new(input.as_bytes()), &mut output).unwrap();
        assert_eq!(input, String::from_utf8(output).unwrap());
    }

    #[test]
    fn two_profiles_first_default() {
        let input = r#"[Install0123456789ABCDEF]
Default=abcdefgh.default
Locked=1

[Profile0]
Name=default
Path=abcdefgh.default
# we shift this line below the blank line, which is irritating but harmless
Default=1

[Profile1]
Name=other
Path=zyxwvuts.other
"#;
        let mut output = Vec::new();
        read_update_write(BufReader::new(input.as_bytes()), &mut output).unwrap();
        let expected = r#"[Install0123456789ABCDEF]
Default=abcdefgh.default
Locked=1

[Profile0]
Name=default
Path=abcdefgh.default
# we shift this line below the blank line, which is irritating but harmless

Default=1
[Profile1]
Name=other
Path=zyxwvuts.other
"#;
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn two_profiles_second_default() {
        let input = r#"[Install0123456789ABCDEF]
# the fact that we don’t touch this line is actually a bug
Default=zyxwvuts.other
Locked=1

[Profile0]
Name=default
Path=abcdefgh.default

[Profile1]
Name=other
Path=zyxwvuts.other
Default=1
"#;
        let mut output = Vec::new();
        read_update_write(BufReader::new(input.as_bytes()), &mut output).unwrap();
        let expected = r#"[Install0123456789ABCDEF]
# the fact that we don’t touch this line is actually a bug
Default=zyxwvuts.other
Locked=1

[Profile0]
Name=default
Path=abcdefgh.default

Default=1
[Profile1]
Name=other
Path=zyxwvuts.other
"#;
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
