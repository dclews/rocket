# rocket
Cargo sub-command for building and running bare-metal crates.

# Building on Windows
Using rustup and MSYS2 is probably the easiest way to build on Windows.

Depdencies can be installed via:
  ```sh
  pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-cmake mingw-w64-x86_64-make git
  ```
where "w64-x86_64" can be replaced with the target architecture.

Then cargo and rust can be added to the path through ~/.bash_profile
```sh
export PATH="$USERPOFILE/.cargo/bin:$PATH"
```
