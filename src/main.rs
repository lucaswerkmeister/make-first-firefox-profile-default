use anyhow::{anyhow, Context, Result};
use chrono::Local;
use dirs::home_dir;
use std::fs::{rename, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;

fn profiles_ini_path() -> Result<(PathBuf, PathBuf)> {
    let mut dir_path = home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    dir_path.push(".mozilla");
    dir_path.push("firefox");
    let mut file_path = dir_path.clone();
    file_path.push("profiles.ini");
    Ok((dir_path, file_path))
}

fn first_profile_path(profiles_ini: &str) -> Result<&str> {
    let mut lowest_profile: Option<(u64, Option<&str>)> = None;
    let mut in_lowest_profile = false;
    for line in profiles_ini.lines() {
        if line.starts_with("[") {
            in_lowest_profile = false;
            if line.starts_with("[Profile") && line.ends_with("]") {
                let current_num = &line["[Profile".len()..(line.len() - "]".len())];
                let current_num: u64 = current_num
                    .parse()
                    .context("Unable to parse profile number")?;
                if lowest_profile.is_none_or(|(lowest_num, _)| current_num < lowest_num) {
                    lowest_profile = Some((current_num, None));
                    in_lowest_profile = true;
                }
            }
        } else if in_lowest_profile && line.starts_with("Path=") {
            if let Some((_, path)) = &mut lowest_profile {
                *path = Some(&line["Path=".len()..]);
            }
        }
    }
    lowest_profile
        .ok_or_else(|| anyhow!("No profile section found"))?
        .1
        .ok_or_else(|| anyhow!("No path found in first profile section"))
}

fn write_profiles_ini<W: Write>(input: &str, mut writer: W) -> Result<()> {
    let mut in_install_section = false;
    for line in input.lines() {
        if line.starts_with("[") {
            in_install_section = line.starts_with("[Install");
            writeln!(writer, "{}", line)?;
        } else if in_install_section && line.starts_with("Default=") {
            writeln!(writer, "Default={}", first_profile_path(input)?)?;
        } else {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let (dir_path, file_path) = profiles_ini_path()?;
    let mut input_file = File::open(&file_path)?;
    let mut input = String::new();
    input_file
        .read_to_string(&mut input)
        .context("Unable to read profiles.ini")?;
    let tmp_path = dir_path.join(Local::now().format("profiles.ini-%+").to_string());
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)
        .context("Unable to create temporary output file")?;
    write_profiles_ini(&input, BufWriter::new(output_file))?;
    rename(tmp_path, file_path)
        .context("Unable to turn temporary output file into real profiles.ini")?;

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
        write_profiles_ini(&input, &mut output).unwrap();
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
Default=1

[Profile1]
Name=other
Path=zyxwvuts.other
"#;
        let mut output = Vec::new();
        write_profiles_ini(&input, &mut output).unwrap();
        let expected = r#"[Install0123456789ABCDEF]
Default=abcdefgh.default
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

    #[test]
    fn two_profiles_second_default() {
        let input = r#"[Install0123456789ABCDEF]
Default=zyxwvuts.other
Locked=1

[Profile0]
Name=default
Path=abcdefgh.default

[Profile1]
Name=other
Path=zyxwvuts.other
# we don’t touch this line – it appears to have no effect
Default=1
"#;
        let mut output = Vec::new();
        write_profiles_ini(&input, &mut output).unwrap();
        let expected = r#"[Install0123456789ABCDEF]
Default=abcdefgh.default
Locked=1

[Profile0]
Name=default
Path=abcdefgh.default

[Profile1]
Name=other
Path=zyxwvuts.other
# we don’t touch this line – it appears to have no effect
Default=1
"#;
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
