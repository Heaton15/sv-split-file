# sv-split-file

This CLI tool receives a input .v / .sv file or directory which .v / .sv files and splits all modules inside of them into their own files.

# Build

You can build this tool with the following:

```
git clone git@gitlab-ext.galois.com:niobium/personal-projects/tim/sv-split-file.git
cd sv-split-file/
cargo build --release
```

A `sv-split-file` binary will then exist in the `target/release` directory.


# Instructions


The first argument is the input file or path, and the second arguemnt is where to place the results

E.G

```
target/release/sv-split-file my_file.sv new_dir/
```

Feel free to move the `sv-split-file` binary to somewhere in your PATH.


# sv-split-file
