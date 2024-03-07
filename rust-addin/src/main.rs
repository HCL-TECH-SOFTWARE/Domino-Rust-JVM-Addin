//
// Copyright (c) 2023-2024 HCL America, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use anyhow::{anyhow, Context};
use glob::{glob_with, MatchOptions};
use jni::{
    objects::{JObject, JString, JValue},
    InitArgsBuilder, JNIVersion, JavaVM,
};
use std::{
    borrow::Borrow,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};
mod notes;

const NAME: &str = "Rust Addin";
const ENTRY_CLASS: &str = "com/hcl/example/addin/ExampleAddin";
const QUEUE_NAME: &str = "MQ$RUSTADDIN";

const DEFAULTJAVADEBUGCMD: &str =
    "-agentlib:jdwp=transport=dt_socket,server=y,address=0.0.0.0:8000,suspend=y";

const JAVAUSEROPTIONSFILE: &str = "JavaUserOptionsFile";
const RUSTADDINJAVAUSEROPTIONSFILE: &str = "RustAddinJavaUserOptionsFile";

/// optional - where the addin JARs are installed
const RUSTADDINDIR: &str = "RustAddinInstallDir";
/// optional - set up for addin debug
const RUSTADDINDEBUG: &str = "RustAddinDebug";
/// optional - set up for Java debug
const RUSTADDINJAVADEBUG: &str = "RustAddinJavaDebug";
/// optional - if set, will override the standard java debug command
const RUSTADDINJAVADEBUGCMD: &str = "RustAddinJavaDebugCmd";
/// optional - java home, will use {dominoBin}/jvm if not set
const RUSTADDINJAVAHOME: &str = "RustAddinJavaHome";
/// optional - Number of seconds to wait before exec Java command, RHEL has a problem loading
///            if run on ServerTasks.
const RUSTADDINLAUNCHWAITSECS: &str = "RustAddinLaunchWaitSecs";
/// optional - Set the Java temp directory when launching the JVM
const RUSTADDINTEMPDIRECTORY: &str = "RustAddinTempDirectory";
/// optional - set the Java heap size in mb
const RUSTADDINJAVAHEAPINMB: &str = "RustAddinJavaHeapInMB";
/// optional - add if you want additional Java params
const RUSTADDINADDITIONALJAVAPARAMS: &str = "RustAddinAdditionalJavaParams";

// set up standard Java settings
// Java 8 uses UseStringDeduplicationJVM and a later version of Java 8 and on uses UseStringDeduplication.
// after UseStringDeduplicationJVM is gone, JVM will give an error and not start,
// so use IgnoreUnrecognizedVMOptions to not error.  Instead of trying to determine correct version, pass both
const STANDARD_JVM_OPTS: &[&str] = &[
    "-XX:+HeapDumpOnOutOfMemoryError",
    "-XX:+UseG1GC",
    "-XX:+UseStringDeduplicationJVM",
    "-XX:+UseStringDeduplication",
    "-XX:+IgnoreUnrecognizedVMOptions"
];

fn main() {
    notes::init(std::env::args());

    // Exit early if the process is already running
    if notes::queue_exists(QUEUE_NAME) {
        log("Already running");
        notes::term();
        return;
    }

    log("Starting...");

    let debug = check_ini_bool(RUSTADDINDEBUG);

    if debug {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    // Find our executable directory
    let exec_dir = notes::exec_dir();

    // Find our data directory
    let data_dir: PathBuf = notes::data_dir();

    let install_dir;
    match locate_addin(&exec_dir) {
        Ok(dir) => install_dir = dir,
        Err(e) => {
            log(format!("Encountered error initializing {}: {}", NAME, e));
            notes::term();
            return;
        }
    }

    if debug {
        log(format!(
            "Initializing with executable directory {}, data directory {}, and {} directory {}",
            exec_dir.display(),
            data_dir.display(),
            NAME,
            install_dir.display()
        ));
    }

    // Standardize paths
    std::env::set_var("NOTESDATA", data_dir.display().to_string());
    std::env::set_var("NOTESROOT", exec_dir.display().to_string());
    std::env::set_var("Notes_ExecDirectory", exec_dir.display().to_string());
    std::env::set_current_dir(&data_dir).unwrap();
    
    // Ensure that the program directory is in the path for Windows lib loading
    let mut path_env = std::env::var("PATH").to_owned().unwrap_or("".to_owned());
    path_env.push_str(path_sep());
    path_env.push_str(exec_dir.display().to_string().as_str());
    std::env::set_var("PATH", path_env);

    match run_addin(debug, exec_dir, install_dir) {
        Ok(_) => {}
        Err(e) => log(format!("Encountered error running {}: {}", NAME, e)),
    }

    log("Shutdown");

    notes::term();
}

fn run_addin(debug: bool, exec_dir: PathBuf, install_dir: PathBuf) -> anyhow::Result<()> {
    // Tell the JNI init to use Domino's JVM unless otherwise specified
    let java_home_custom = notes::get_ini_var(RUSTADDINJAVAHOME).unwrap_or("".to_owned());
    if !java_home_custom.is_empty() {
        std::env::set_var("JAVA_HOME", java_home_custom);
    } else {
        std::env::set_var("JAVA_HOME", exec_dir.join("jvm").display().to_string());
    }

    if debug {
        log(format!(
            "Using JAVA_HOME {}",
            std::env::var("JAVA_HOME").unwrap_or_default()
        ));
    }

    // Check to see if we have a configured wait duration
    let wait_secs_var = notes::get_ini_var(RUSTADDINLAUNCHWAITSECS).unwrap_or("".to_owned());
    if !wait_secs_var.is_empty() {
        let wait_secs = std::time::Duration::from_secs(wait_secs_var.parse::<u64>().unwrap_or(0));
        if wait_secs.as_secs() > 0 {
            std::thread::sleep(wait_secs);
        }
    }

    // Look for a Java options file setting
    let opts_file = notes::get_ini_var(RUSTADDINJAVAUSEROPTIONSFILE)
        .or_else(|| notes::get_ini_var(JAVAUSEROPTIONSFILE))
        .unwrap_or("".to_owned());
    let mut extra_opts: Vec<String> = Vec::new();
    if !opts_file.is_empty() {
        if let Ok(lines) = read_lines(opts_file) {
            lines
                .flatten()
                .map(|line| {
                    if line.starts_with("-") {
                        line.to_string()
                    } else {
                        format!("-{}", line.to_string())
                    }
                })
                .for_each(|line| extra_opts.push(line.clone()));
        }
    }

    // Initialize our JVM with the classpath and get an environment tied to our thread
    let mut jvm_args_builder = InitArgsBuilder::new().version(JNIVersion::V8);

    for opt in STANDARD_JVM_OPTS {
        jvm_args_builder = jvm_args_builder.option(opt.to_owned());
    }

    // Set a custom temp directory if specified
    if let Some(temp_dir) = notes::get_ini_var(RUSTADDINTEMPDIRECTORY) {
        jvm_args_builder = jvm_args_builder.option(format!("-Djava.io.tmpdir={}", temp_dir));
    }

    // If heap size is set, add it to the Java line
    if let Some(heap_size) = notes::get_ini_var(RUSTADDINJAVAHEAPINMB) {
        jvm_args_builder = jvm_args_builder.option(format!("-Xmx{}m", heap_size));
    }

    // Add in flat options from the INI, if present
    if let Some(ini_opts) = notes::get_ini_var(RUSTADDINADDITIONALJAVAPARAMS) {
        match shell_words::split(ini_opts.as_str()) {
            Ok(opts) => {
                for opt in opts {
                    jvm_args_builder = jvm_args_builder.option(opt);
                }
            }
            Err(e) => {
                log(format!(
                    "Unable to parse JVM arguments \"{}\": {}",
                    ini_opts, e
                ));
            }
        }
    }

    // Add in extra options read from a file, if present
    for opt in extra_opts {
        jvm_args_builder = jvm_args_builder.option(opt);
    }

    // If debugging Java, set up Java debug command
    if check_ini_bool(RUSTADDINJAVADEBUG) {
        let java_debug_cmd =
            notes::get_ini_var(RUSTADDINJAVADEBUGCMD).unwrap_or(DEFAULTJAVADEBUGCMD.to_owned());
        jvm_args_builder = jvm_args_builder.option(java_debug_cmd);
    }

    if debug {
        log("Launching JVM with arguments:");
        for arg in jvm_args_builder.options().unwrap() {
            log(format!("* {:?}", arg));
        }
    }

    jvm_args_builder = jvm_args_builder.option(build_classpath_arg(debug, &exec_dir, &install_dir));
    let jvm_args = jvm_args_builder.build()?;
    let jvm = JavaVM::new(jvm_args)
      .with_context(|| "Encountered exception initializing JVM")?;

    let mut env = jvm.attach_current_thread()?;

    // Create a String[] from the addin's argv
    let string_class = env.find_class("java/lang/String")?;
    let args = std::env::args()
        .map(|arg| env.new_string(arg).unwrap())
        .collect::<Vec<JString>>();
    let argv = env
        .new_object_array(
            args.len().try_into().unwrap(),
            string_class,
            JObject::null(),
        )
        .unwrap();
    for (i, arg) in args.iter().enumerate() {
        env.set_object_array_element(argv.borrow(), i.try_into().unwrap(), arg)?;
    }

    // Call ExampleAddin.main(argv)
    let argv_obj: JObject = JObject::from(argv);
    let main_class = env
        .find_class(ENTRY_CLASS)
        .with_context(|| format!("Could not find entrypoint class {}", ENTRY_CLASS))?;
    env.call_static_method(
        main_class,
        "main",
        "([Ljava/lang/String;)V",
        &[JValue::from(&argv_obj)],
    ).context(format!("Encountered exception running {}", NAME))?;

    return Ok(());
}

/// Builds a classpath argument to pass to the JVM.
///
/// This builder searches the "rustaddin", "ndext", and "jvm/lib/ext"
/// directories within the Domino program directory for JARs to
/// include, explicitly excluding a few known-unwanted ones
fn build_classpath_arg(debug: bool, notes_dir: &PathBuf, install_dir: &PathBuf) -> String {
    let mut classpath = Vec::new();

    add_jars(debug, install_dir, "*.jar", &mut classpath);
    add_jars(debug, notes_dir, "ndext/*.jar", &mut classpath);
    add_jars(debug, notes_dir, "jvm/lib/ext/*.jar", &mut classpath);

    let sep = path_sep();

    let mut cp = "-Djava.class.path=".to_owned();
    cp.push_str(classpath.join(sep).as_str());

    return cp;
}

/// Adds all JARs matching the given filesystem glob pattern to the
/// provided list, making the paths relative to the current execution
/// directory
fn add_jars(debug: bool, search_dir: &PathBuf, pattern: &str, classpath: &mut Vec<String>) {
    let mut glob_path = search_dir.display().to_string().to_owned();
    if !glob_path.ends_with("/") {
        glob_path.push_str("/");
    }
    glob_path.push_str(pattern);

    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    for jar in glob_with(glob_path.as_str(), options).unwrap() {
        if let Ok(path) = jar {
            if !(path.ends_with("jsdk.jar") || path.ends_with("guava.jar")) {
                if debug {
                    log(format!("Adding classpath entry {}", path.display()));
                }
                classpath.push(path.display().to_string());
            }
        }
    }
}

/// Retrieves the named value and determines whether it is set to "true" or "1"
fn check_ini_bool(var_name: &str) -> bool {
    let val = notes::get_ini_var(var_name).unwrap_or("".to_owned());
    return val.eq_ignore_ascii_case("true") || val.eq("1");
}

fn locate_addin(exec_dir: &PathBuf) -> anyhow::Result<PathBuf> {
    let path = notes::get_ini_var(RUSTADDINDIR)
        .and_then(|val| {
            if val.is_empty() {
                return Option::None;
            } else {
                return Option::Some(val);
            }
        })
        .map(|path| Path::new(path.as_str()).to_path_buf())
        .unwrap_or_else(|| {
            let mut exec = exec_dir.clone();
            exec.push("rustaddin-lib");
            return exec;
        });
    if path.exists() && path.is_dir() {
        return Ok(path);
    } else {
        return Err(anyhow!("Unable to locate the {} installation in {}. Ensure that the {} notes.ini property points to the correct location", NAME, path.display(), RUSTADDINDIR));
    }
}

fn log<N: AsRef<str>>(msg: N) {
    let formatted = format!("{}: {}", NAME, msg.as_ref());
    notes::addin_log(formatted);
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn path_sep() -> &'static str {
    if std::env::consts::OS.eq("windows") {
        ";"
    } else {
        ":"
    }
}
