use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::fs;
use std::process;
use git2;
use regex;

pub enum Source {
    Local(String),
    Git(String),
}
impl Source {
    pub fn from_str(s: &str) -> Source {
        let source_url_regex = regex::Regex::new(r"@(.+:.+)")
            .expect("Failed to build regex for source parsing");

        if source_url_regex.is_match(s) {
            Source::Git(s.to_owned())
        }
        else {
            Source::Local(s.to_owned())
        }
    }
}

pub enum Target {
    BuiltIn(String),
    Json(String),
}
impl Target {
    pub fn from_str(s: &str) -> Target {
        if s.ends_with(".json") {
            Target::Json(s.to_owned())
        }
        else {
            Target::BuiltIn(s.to_owned())
        }
    }
    pub fn as_string(&self) -> String {
        match *self {
            Target::BuiltIn(ref s) => s.clone(),
            Target::Json(ref s) => {
                let p = Path::new(s.as_str());
                let parent = p.parent().unwrap();
                let stem = p.file_stem().unwrap();
                let mut pb = parent.to_path_buf();
                pb.push(stem);
                String::from(pb.to_str().unwrap())
            }
        }
    }
    pub fn as_stem(&self) -> String {
        match *self {
            Target::BuiltIn(ref s) => s.clone(),
            Target::Json(ref s) => {
                let p = Path::new(s.as_str());
                String::from(p.file_stem().unwrap().to_str().unwrap())
            },
        }
    }
}

pub enum Feature {
    Common(String),
    ArchSpecific(String),
}
impl Feature {
    pub fn canonicalize(&self, t: &Target) -> String {
        match *self {
            Feature::Common(ref s) => {
                s.clone()
            },
            Feature::ArchSpecific(ref s) => {
                format!("_{}-{}", t.as_stem(), s)
            },
        }
    }
}
#[derive(Debug)]
pub enum Loader {
    GRUB,
}
impl Loader {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "grub" => Ok(Loader::GRUB),
            _ => Err(format!("Unknown bootloader {}", s)),
        }
    }
}
pub enum Runner {
    Qemu,
}
impl Runner {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "qemu" => Ok(Runner::Qemu),
            _ => Err(format!("Unknown bootloader {}", s)),
        }
    }
}
pub struct Payload {
    source: Option<Source>,
    target: Target,
    loader: Option<Loader>,
    runner: Option<Runner>,
    features: Vec<Feature>,
    path: PathBuf,
}

impl Payload {
    pub fn new(options: Vec<String>) -> Result<Payload, String> {
        let mut source: Option<Source> = None;
        let mut target: Option<Target> = None;
        let mut loader: Option<Loader> = None;
        let mut runner: Option<Runner> = None;
        let mut features: Vec<Feature> = vec![];
        let path = Path::new("./payload").to_owned();

        let option_regex = regex::Regex::new(r"--(\w+)=(\S+)")
            .expect("Failed to build regex for option parsing");

        for o in options {
            let o = o.as_str();
            if !option_regex.is_match(o) {
                return Err(format!("Unrecognized option {}", o));
            }
            for cap in option_regex.captures_iter(o) {
                let option_name = cap.at(1).unwrap();
                let option_value = cap.at(2).unwrap();

                match option_name {
                    "source" => source = Some(Source::from_str(option_value)),
                    "target" => target = Some(Target::from_str(option_value)),
                    "loader" => loader = Some(Loader::from_str(option_value).unwrap()),
                    "runner" => runner = Some(Runner::from_str(option_value).unwrap()),
                    _ => return Err(format!("Unrecognised payload option {}", option_name)),
                };
            }
        }

        let target = match target {
            Some(t) => t,
            None => return Err("No target configured".to_owned()),
        };
        Ok(Payload {
            source: source,
            target: target,
            loader: loader,
            runner: runner,
            features: features,
            path: path,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn clean(&mut self) {
        let payload_path = self.path();
        if payload_path.exists() {
            Self::say("Clean",
                      format!("Removing payload directory {}",
                              payload_path.to_str().expect("Invalid path")));
            let _ = fs::remove_dir_all(payload_path);
        }
    }
    pub fn fetch(&mut self) -> Result<PathBuf, String> {
        let payload_path = self.path();
        match self.source {
            Some(Source::Local(_)) => Ok(payload_path.to_path_buf()),
            Some(Source::Git(ref url)) => {
                    let mut cb = git2::RemoteCallbacks::new();
                    let _ = cb.credentials(Payload::auth);

                    let mut co = git2::build::CheckoutBuilder::new();

                    let mut fo = git2::FetchOptions::new();
                    let _ = fo.remote_callbacks(cb);

                    let repo = match git2::build::RepoBuilder::new()
                        .fetch_options(fo)
                        .with_checkout(co)
                        .clone(url.as_str(), payload_path) {
                        Ok(repo) => {
                            println!("Fetched remote repository {:?}", url);
                            repo
                        }
                        Err(e) => return Err(format!("Failed to clone payload git repo: {}", e)),
                    };
                    Ok(payload_path.to_path_buf())
                },
            None => Err("No source configured from which to fetch".to_owned()),
        }
    }
    fn auth(url: &str,
            username: Option<&str>,
            cred_types: git2::CredentialType)
            -> Result<git2::Cred, git2::Error> {
        //        let mut cred_helper = git2::CredentialHelper::new(url);
        let username = match username {
            Some(username) => username.to_owned(),
            None => {
                match Self::ask("Auth", format!("Enter username for {}", url)) {
                    Ok(username) => username,
                    Err(_) => return Err(git2::Error::from_str("Failed to read username")),
                }
            }
        };
        println!("Username is {}", username);

        if cred_types.contains(git2::USER_PASS_PLAINTEXT) {
            match Self::ask("Auth", format!("Enter password for {}", url)) {
                Ok(password) => {
                    println!("Password is {}", password);
                    Ok(git2::Cred::userpass_plaintext(username.as_str(), password.as_str())
                        .expect("failed to create credential object"))
                }
                Err(_) => return Err(git2::Error::from_str("Failed to read password")),
            }
        } else {
            Err(git2::Error::from_str("Failed to authenticate"))
        }
    }
    pub fn doc(&mut self) -> Result<(), String> {
        self.cargo("cargo", "doc")
    }
    pub fn build(&mut self) -> Result<(), String> {
        self.cargo("xargo", "build")
    }
    pub fn run(&mut self) -> Result<(), String> {
        self.cargo("xargo", "run")
    }
    fn cargo(&mut self, command: &str, cargo_subcommand: &str) -> Result<(), String> {
        let target = self.target.as_string();

        let mut args: Vec<String> = vec![];
        args.push(cargo_subcommand.to_owned());
        args.push("--verbose".to_owned());
        args.push("--target".to_owned());
        args.push(target);

        args.push("--features".to_owned());
        self.features.push(Feature::ArchSpecific("rt".to_owned()));
        match self.loader {
            Some(Loader::GRUB) => {
                self.features.push(Feature::ArchSpecific("grub".to_owned()));
            }
            None => {}
        };
        let feature_strings: Vec<String> = self.features
            .iter()
            .map(|f| f.canonicalize(&self.target))
            .collect();

        let feature_string = feature_strings.join(" ");
        args.push(feature_string);

        println!("Running {} {}", command, args.join(" "));
        let mut child = process::Command::new(command)
            .args(args.as_slice())
            .spawn()
            .expect("Failed to execute build system");

        match child.wait() {
            Ok(status) => {
                if status.success() {
                    println!("Build successful");
                    Ok(())
                } else {
                    Err(format!("Build failed. Error code {}", status))
                }
            }
            Err(e) => Err(format!("Failed to execute xargo: {}", e)),
        }
    }
    fn say(stage: &str, s: String) {
        print!("[{}] {}", stage, s);
    }
    fn ask(stage: &str, question: String) -> io::Result<String> {
        let mut answer = String::new();
        Self::say(stage, question);
        let _ = io::stdout().flush();
        match io::stdin().read_line(&mut answer) {
            Ok(_) => Ok(answer),
            Err(e) => Err(e),
        }
    }
}
