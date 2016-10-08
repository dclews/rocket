# rocket
Cargo sub-command for building and running bare-metal crates. Requires [xargo](https://github.com/japaric/xargo).

Application commands take the form `-xyz`, with each command being executed in order. Basic commands are:
* b => build payload
* c => clean payload
* f => fetch payload
* d => build payload documentation
* r => build and run payload using a specified loader

In addition to that payloads can be further configured with long form options taking the form "--option=val{val_args}" where val and val_args are optional. Basic options are:
* `--target=JSON_TARGET_SPEC|BUILTIN_TARGET` - build the payload for the specified LLVM target.
* `--loader=BOOTLOADER_NAME` - build support for the specified bootloader.
* `--runner=RUNNER_NAME{RUNNER_ARGUMENTS}` - use the specified runner to run the generated payload artefact.
* `--debug_print=true|false` - enable debug! printing feature in payload.

A standard invocation to build a crate for x86_64 with the GRUB bootloader and run it with qemu:
`$ cargo rocket -r --target=x86_64 --loader=grub --runner=qemu`
