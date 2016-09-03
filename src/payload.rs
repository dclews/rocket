use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::fs;
use std::process;
use git2;

pub enum Source<'a> {
    Local(&'a Path),
    Git(&'a str),
}

pub enum Target<'a> {
    BuiltIn(&'a str),
    LocalJson(&'a Path),
    RemoteJson(&'a str),
}
impl<'a> Target<'a> {
    pub fn as_string(&self) -> String {
        match *self {
            Target::BuiltIn(s) => s.to_owned(),
            Target::LocalJson(p) => {
                let parent = p.parent().unwrap();
                let stem = p.file_stem().unwrap();
                let mut pb = parent.to_path_buf();
                pb.push(stem);
                String::from(pb.to_str().unwrap())
            },
            Target::RemoteJson(p) => {
                panic!("Not implemented");
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
            Feature::Common(ref s) => s.clone(),
            Feature::ArchSpecific(ref s) => format!("_{}-{}", t.as_string(), s),
        }
    }
}
#[derive(Debug)]
pub enum Loader {
    GRUB,
}
pub enum Runner {
    Qemu,
}

pub struct Payload<'a> {
    source: Source<'a>,
    target: Target<'a>,
    features: Vec<Feature>,
    loader: Option<Loader>,
    runner: Option<Runner>,
    path: PathBuf,
}

impl<'a> Payload<'a> {
    pub fn new(source: Source<'a>,
               target: Target<'a>,
               features: Vec<Feature>,
               loader: Option<Loader>,
               runner: Option<Runner>)
               -> Result<Payload<'a>, String> {
        Ok(Payload {
            path: Path::new("./payload").to_owned(),
            source: source,
            target: target,
            features: features,
            loader: loader,
            runner: runner,
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
            Source::Local(_) => Ok(payload_path.to_path_buf()),
            Source::Git(url) => {
                let mut cb = git2::RemoteCallbacks::new();
                let _ = cb.credentials(Payload::auth);

                let mut co = git2::build::CheckoutBuilder::new();

                let mut fo = git2::FetchOptions::new();
                let _ = fo.remote_callbacks(cb);

                let repo = match git2::build::RepoBuilder::new()
                    .fetch_options(fo)
                    .with_checkout(co)
                    .clone(url, payload_path) {
                    Ok(repo) => {
                        println!("Fetched remote repository {:?}", url);
                        repo
                    }
                    Err(e) => return Err(format!("Failed to clone payload git repo: {}", e)),
                };
                Ok(payload_path.to_path_buf())
            }
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
                    Err(e) => return Err(git2::Error::from_str("Failed to read username")),
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
                Err(e) => return Err(git2::Error::from_str("Failed to read password")),
            }
        } else {
            Err(git2::Error::from_str("Failed to authenticate"))
        }
    }
    pub fn doc(&mut self) -> Result<(), String> {
        Err(String::from("Unimplemented"))
    }
    pub fn build(&mut self) -> Result<(), String> {
        let target = self.target.as_string();
        let mut xargo = process::Command::new("xargo");
        let _ = xargo.current_dir(Path::new("./payload"));

        let _ = xargo.arg("--target");
        let _ = xargo.arg(target);
        let _ = xargo.arg("--features=");

        self.features.push(Feature::ArchSpecific("rt".to_owned()));
        match self.loader {
            Some(Loader::GRUB) => {
                self.features.push(Feature::ArchSpecific("grub".to_owned()));
            },
            None => {},
        };

        for f in self.features.iter() {
            println!("{}", f.canonicalize(&self.target));
            xargo.arg(f.canonicalize(&self.target));
        }

        println!("Running xargo");
        let output = xargo.spawn().expect("Failed to execute xargo");
        Ok(())
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
