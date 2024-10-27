use anyhow::{anyhow, Context, Result};
use dirs;
use std::fs::{rename, File, OpenOptions};
use std::io::{stdout, BufWriter, Read, Write};
use std::path::PathBuf;

fn profiles_ini_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or_else(|| anyhow!("Unable to determine home directory"))?;
    path.push(".mozilla");
    path.push("firefox");
    path.push("profiles.ini");
    Ok(path)
}

fn first_profile_path(profiles_ini: &str) -> &str {
    let mut lowest_profile: Option<(u64, Option<&str>)> = None;
    let mut in_lowest_profile = false;
    for line in profiles_ini.lines() {
        if line.starts_with("[Profile") {
            in_lowest_profile = false;
            let current_num = &line["[Profile".len()..(line.len() - "]".len())];
            let current_num: u64 = current_num.parse().unwrap();
            if let Some((lowest_num, _)) = lowest_profile {
                if current_num < lowest_num {
                    lowest_profile = Some((current_num, None));
                    in_lowest_profile = true;
                }
            } else if lowest_profile.is_none() {
                lowest_profile = Some((current_num, None));
                in_lowest_profile = true;
            }
        } else if line.starts_with("[") {
            in_lowest_profile = false;
        } else if line.starts_with("Path=") && in_lowest_profile {
            if let Some((_, path)) = &mut lowest_profile {
                *path = Some(&line["Path=".len()..]);
            }
        }
    }
    lowest_profile.unwrap().1.unwrap()
}

fn read_update_write<W: Write>(input: &str, mut writer: W) -> Result<()> {
    let mut in_install_section = false;
    for line in input.lines() {
        if line.starts_with("[") {
            in_install_section = line.starts_with("[Install");
            writeln!(writer, "{}", line)?;
        } else if in_install_section && line.starts_with("Default=") {
            writeln!(writer, "Default={}", first_profile_path(input))?;
        } else {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut input_file = File::open(&profiles_ini_path()?)?;
    let mut input = String::new();
    input_file
        .read_to_string(&mut input)
        .context("Unable to read profiles.ini")?;
    read_update_write(&input, stdout())?;

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
        read_update_write(&input, &mut output).unwrap();
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
        read_update_write(&input, &mut output).unwrap();
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
        read_update_write(&input, &mut output).unwrap();
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
