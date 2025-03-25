use std::io::prelude::*;
use std::{
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

use regex::Regex;

const FILETYPES: [&str; 2] = ["sv", "v"];

/// This struct contains a string to a verilog file
#[derive(Clone)]
pub struct SvFile {
    file: String,
}

impl SvFile {
    /// Creates a new instance of a SvFile
    pub fn new(file: PathBuf) -> Vec<SvFile> {
        vec![
            (Self {
                file: SvFile::validate(file),
            }),
        ]
    }

    /// Internal builder to create SvFile types without validation
    fn build(file: String) -> Self {
        Self { file }
    }

    /// Parses the file and dumps the contents to the output_dir
    pub fn dump(&self) {
        todo!("Implement a way to dump the SvFile")
    }

    /// Validate that the input PathBuf is a verilog file
    fn validate(file: PathBuf) -> String {
        let string = file
            .to_str()
            .unwrap_or_else(|| panic!("Input string {} is not valid UTF-8", file.display()));

        // Crash if there is no file extension
        if file.extension().is_none() {
            panic!(
                "Input file {} does not contain an extension, which should be of the following format: {:?}",
                file.display(),
                FILETYPES
            );
        }

        // Crash if the extension does not match the expected types
        if let Some(ext) = file.extension() {
            let ext = ext.to_str().unwrap().to_string();
            if !FILETYPES.contains(&ext.as_str()) {
                panic!(
                    "Extension {} must be one of the following: {:?}:",
                    ext, FILETYPES,
                );
            }
        }

        string.to_string()
    }
}

/// This struct contains a string to a directory which should contain verilog files
pub struct SvDir {}

impl SvDir {
    pub fn build(dir: PathBuf) -> Vec<SvFile> {
        let paths = fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{e}: Error parsing input directory {}", dir.display()));

        let mut file_list: Vec<SvFile> = Vec::new();

        for item in paths {
            match item {
                Ok(d) => {
                    if let Ok(pb) = SvDir::is_sv_file(d) {
                        file_list.push(SvFile::build(format!("{}/{}", dir.to_str().unwrap(), pb)))
                    }
                }
                Err(_) => continue,
            }
        }
        file_list
    }

    fn is_sv_file(entry: fs::DirEntry) -> Result<String, Box<dyn Error>> {
        let sv_file_result = PathBuf::from(entry.file_name())
            .to_str()
            .expect("Not valid UTF-8 string")
            .to_string();

        let ext = PathBuf::from(entry.file_name())
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .ok_or(format!("Entry {:?} does not have a file extension", entry))?;

        if FILETYPES.contains(&ext.as_str()) {
            Ok(sv_file_result)
        } else {
            Err(format!("Entry {:?} is not a SV file", entry).into())
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct ModuleName {
    name: String,
}

struct ModuleBody {
    body: String,
}

pub fn process_files(file_list: Vec<SvFile>, output_dir: PathBuf) {
    // If we have no files to process, there's nothing to run on
    if file_list.is_empty() {
        panic!("No SV or V files were found for splitting. Exiting Now");
    }

    // Create the output directory if it does not exist
    if !fs::metadata(&output_dir)
        .map(|r| r.is_dir())
        .unwrap_or(false)
    {
        fs::create_dir(&output_dir).unwrap_or_else(|e| {
            panic!(
                "Errror {e}: Problem creating directory {}",
                output_dir.display()
            )
        })
    }

    // Regex that matches the module name
    let re = Regex::new(r"^module\s+(\w+)").unwrap();

    // Track module names. Overlapping module names should cause a crash
    //let mut db: HashMap<ModuleName, ModuleBody> = HashMap::new();
    let mut db: Vec<(ModuleName, ModuleBody)> = Vec::new();

    let mut module_is_next = true;
    for sv_file in file_list {
        // Get a line iterator over the entire file
        let file_contents = BufReader::new(
            File::open(&sv_file.file)
                .unwrap_or_else(|e| panic!("Error {e}: Unable to open file {}", sv_file.file)),
        )
        .lines();

        let mut buffer = String::with_capacity(4096);
        for line in file_contents {
            let line = line.unwrap_or_else(|e| {
                panic!(
                    "Error {e}: Unable to read line from file {:?}",
                    &sv_file.file
                )
            });

            let cap = re.captures(&line);

            let module = cap.map(|cap| ModuleName {
                name: cap
                    .get(1)
                    .expect("Module name could not be found")
                    .as_str()
                    .to_string(),
            });

            if module_is_next {
                let Some(module) = module else { continue };
                buffer.clear();
                module_is_next = false;
                if db.iter().any(|(one, _)| one == &module) {
                    panic!(
                        "Error: module name {} has already been parsed. Duplicate module names detected and cannot be split",
                        &module.name,
                    );
                }
                db.push((
                    module,
                    ModuleBody {
                        body: String::new(),
                    },
                ));

                buffer.push_str(&format!("{}\n", &line));
            } else {
                if line == "endmodule" {
                    module_is_next = true;
                    buffer.push_str(&format!("{}\n", &line));
                    db.last_mut().unwrap().1.body = buffer.clone();
                    continue;
                }
                buffer.push_str(&format!("{}\n", &line))
            }
        }
    }
    // Dump the database to new files!
    if !output_dir.exists() {
        fs::create_dir(&output_dir).unwrap_or_else(|_| panic!("Error: Unable to create directory {}", output_dir.display()));
    }

    for (module_name, module_body) in db {
        let output_file = format!("{}/{}.sv", &output_dir.display(), module_name.name);

        // Delete the file if it already exists so that we can generate it again
        if PathBuf::from(&output_file).exists() {
            fs::remove_file(&output_file)
                .unwrap_or_else(|_| panic!("Error: Could not remove file {output_file}"));
        }
        let mut output_filepath = File::create(&output_file)
            .unwrap_or_else(|_| panic!("Error: Could not create file {output_file}"));
        output_filepath
            .write_all(module_body.body.as_bytes())
            .expect("Failed writing module body to module {module_name}");
    }
}
