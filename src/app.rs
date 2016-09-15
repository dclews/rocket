use ::payload;
use regex;
use std::env;

#[derive(Debug)]
pub enum Command {
    Clean,
    Fetch,
    Pull,
    Build,
    Doc,
    Run,
}

pub struct App {
    commands: Vec<Command>,
    payload: payload::Payload,
}
impl App {
    pub fn new(args: Vec<String>) -> Result<App, String> {
        let command_regex = regex::Regex::new(r"\A-[a-z]+")
            .expect("Failed to build regex for command parsing");
        let mut commands: Vec<Command> = vec![];
        let mut payload_args: Vec<String> = vec![];
        for arg in args {
            if command_regex.is_match(arg.as_str()) {
                for c in arg.chars().skip(1) {
                    match c {
                        'c' => commands.push(Command::Clean),
                        'f' => commands.push(Command::Fetch),
                        'p' => commands.push(Command::Pull),
                        'b' => commands.push(Command::Build),
                        'd' => commands.push(Command::Doc),
                        'r' => commands.push(Command::Run),
                        _ => return Err(format!("Unrecognized application command '{}'", c)),
                    };
                }
            } else {
                payload_args.push(arg);
            }
        }
        let payload = match payload::Payload::new(payload_args) {
            Ok(payload) => payload,
            Err(e) => return Err(e),
        };
        Ok(
            App{
                commands: commands,
                payload: payload
            }
        )
    }
    pub fn run(&mut self) {
        for command in self.commands.iter() {
            match *command {
                Command::Fetch => {
                    self.payload.fetch().expect("Failed to fetch payload");
                },
                Command::Build => {
                    self.payload.build().expect("Failed to build payload");
                },
                Command::Doc => {
                    self.payload.doc().expect("Failed to document payload");
                },
                Command::Clean => {
                    self.payload.clean();
                },
                Command::Run => {
                    self.payload.run().expect("Failed to run payload");
                }
                _ => {
                    panic!("Command '{:?}' not yet implemented", command);
                },
            };
        }
    }
    pub fn usage() -> String {
"\
Usage: rocket -[COMMAND,...] [PAYLOAD_OPTION_NAME=PAYLOAD_OPTION_VALUE,...]
Example: rocket -br --source=foo@github.com/bar.git --target=x86_64 --loader=grub
Commands:
    (b)uild: Build the local copy of the payload, using any provided options.
    (c)lean: Clean the local copy of the payload.
    (r)un: Run the generated artefact using the specified runner.
    (f)etch: Fetch a payload from a repository.
    (p)ull: Update the local payload from the repository.

Payload Options:
    --source=( url | path )
    --target=( target_name | target_json_path )
    --loader=( grub )
    --runner=( qemu )
".to_owned()
    }
}
