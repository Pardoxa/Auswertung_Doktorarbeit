# Build info

## Already fixed! See [#95](https://github.com/GuillaumeGomez/rust-GSL/issues/95)

Due to a bug in the GSL lib crate, you have to use rustc 1.50.0 or below to compile.
I already submitted an issue there.

Quick fix:

In this foulder execute the command
```bash
rustup override set 1.50.0
```
to manually restrict the ruct compiler version
