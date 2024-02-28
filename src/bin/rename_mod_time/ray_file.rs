use chrono::{DateTime, Local};
use regex::Regex;
use std::{
    cmp::max,
    fs,
    io::{self, Write}, // Write for flush
    iter::zip,
};

// following Unicode standard 15.1.0
// reference: https://en.wikipedia.org/w/index.php?title=CJK_Unified_Ideographs&direction=next&oldid=1203587452
const CHINESE_UNICODE_RANGE: [[u32; 2]; 6] = [
    [0x04_E00, 0x09_FFF], // unified basic chars
    [0x03_400, 0x04_DBF], // extension A
    [0x20_000, 0x2A_6DF], // extension B
    [0x2A_700, 0x2E_E5F], // extensions C, D, E, F, I
    [0x30_000, 0x32_3AF], // extensions G, H
    [0x0F_900, 0x0F_AFF], // round-trip compatibility
                          // [0x03_300, 0x03_3FF], // non-unified chars for legacy systems
                          // [0x0F_E30, 0x0E_F4F], // non-unified chars for legacy systems
                          // [0x0F_900, 0x0F_AFF], // non-unified chars for legacy systems
                          // [0x2F_800, 0x2F_A1F], // non-unified chars for legacy systems
];

pub struct RayFileList {
    file_list: Vec<RayFile>,
    time_format: String,
    max_len_input: usize,
    max_len_output: usize,
}

impl RayFileList {
    pub fn from(input_file_list: &Vec<String>, time_format: String) -> Self {
        let file_list: Vec<RayFile> = input_file_list
            .iter()
            .map(|f| RayFile::from(f.clone()))
            .collect();

        let max_len_input: usize = max(3, file_list.iter().map(|f| f.full_len()).max().unwrap());

        let time_str_len: usize = Local::now().format(&time_format).to_string().len();
        let max_ext_len: usize = file_list.iter().map(|f| f.ext_len()).max().unwrap();
        let max_len_output: usize = max(3, time_str_len + 1 + max_ext_len);

        Self {
            file_list,
            time_format,
            max_len_input,
            max_len_output,
        }
    }

    pub fn rename_with_modification_time(&self, to_print_prompt: bool) {
        let new_file_list: Vec<RayFile> = self.get_renamed_file_list();

        self.print_renaming_header();
        self.print_renaming_operations(&new_file_list);
        if to_print_prompt {
            let to_rename: bool = self.wait_accepting_prompt();
            if !to_rename {
                println!("Nothing done.");
                return;
            }
        }

        // rename files
        zip(&self.file_list, new_file_list).for_each(|(old_file, new_file)| {
            fs::rename(old_file.to_string(), new_file.to_string()).unwrap()
        })
    }

    fn get_renamed_file_list(&self) -> Vec<RayFile> {
        self.file_list
            .iter()
            .map(|f: &RayFile| f.clone().get_renamed_instance(&self.time_format))
            .collect()
    }

    /// return whether to rename or not
    fn wait_accepting_prompt(&self) -> bool {
        let yes_regex: Regex = Regex::new("^[yY]?$").unwrap();
        let no_regex: Regex = Regex::new("^[nN]$").unwrap();
        loop {
            print!("Accept the above renaming? [Y/n] ");
            io::stdout().flush().unwrap();
            let mut buffer: String = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            if yes_regex.is_match(&buffer.trim()) {
                return true;
            }
            if no_regex.is_match(&buffer.trim()) {
                return false;
            }
        }
    }

    fn print_renaming_header(&self) {
        println!(
            "{:^wi$} {:^wo$}",
            "old",
            "new",
            wi = self.max_len_input,
            wo = self.max_len_output
        );
    }

    fn print_renaming_operations(&self, new_list: &Vec<RayFile>) {
        zip(&self.file_list, new_list).for_each(|(o, n)| {
            println!(
                "{:w$} {}",
                o.to_string(),
                n.to_string(),
                w = self.max_len_input - o.get_chinese_length_offset_value()
            )
        });
    }
}

#[derive(Clone)]
pub struct RayFile {
    /// excluding the extension (f_ext)
    f_name: String,
    f_ext: String,
}

impl RayFile {
    pub fn from(f_full_name: String) -> Self {
        if f_full_name.contains("/") {
            todo!("file path in different directory. Please remove all slashes.")
        }

        if f_full_name.starts_with(".") {
            return Self {
                f_name: f_full_name,
                f_ext: String::from(""),
            };
        }

        let ext_dot_position: usize = f_full_name.rfind(".").unwrap();

        RayFile {
            f_name: f_full_name.get(..ext_dot_position).unwrap().to_string(),
            f_ext: f_full_name
                .get((ext_dot_position + 1)..)
                .unwrap()
                .to_string(),
        }
    }

    fn get_renamed_instance(&self, time_format: &String) -> Self {
        // reference: https://doc.rust-lang.org/1.76.0/std/fs/struct.Metadata.html#method.modified
        let metadata: fs::Metadata = fs::metadata(&self.to_string()).unwrap();
        match metadata.modified() {
            Err(err) => panic!("Not supported on this platform.\n{err:?}"),
            Ok(system_time) => {
                let chrono_time: DateTime<Local> = system_time.into();
                Self {
                    f_name: chrono_time.format(time_format).to_string(),
                    f_ext: self.f_ext.clone(),
                }
            }
        }
    }

    fn get_chinese_length_offset_value(&self) -> usize {
        self.f_name
            .chars()
            .map(|c| {
                if c.is_ascii() {
                    0
                } else if CHINESE_UNICODE_RANGE
                    .iter()
                    .any(|r| c as u32 >= r[0] && c as u32 <= r[1])
                {
                    1
                } else {
                    0 // unknown
                }
            })
            .sum()
    }

    fn full_len(&self) -> usize {
        self.f_name.len()
            + if self.f_ext.is_empty() {
                0
            } else {
                1 + self.f_ext.len()
            }
    }

    fn ext_len(&self) -> usize {
        self.f_ext.len()
    }
}

impl std::fmt::Display for RayFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.f_ext.is_empty() {
            write!(f, "{}", self.f_name)
        } else {
            write!(f, "{}.{}", self.f_name, self.f_ext)
        }
    }
}
