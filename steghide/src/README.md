# steghide

A copy of steghide, made in Rust.

This is mostly for learning Rust, steganography, explore GPU use for brute forcing, etc

# TODO

- Hide passphrase from Linux command `ps`
- Handle reading password from stdin
- Handle reading twice password for comfirmation
- Builder Pattern for GetOps (clap crate)

## CLI Requirements

- `--embedfile` can be `-`
- `--extractfile` can be `-`
- `--compress` requires `--embed` , is `i8`, from 1 to 9.
- `--dontcompress` requires `--embed`, sets compression to 0
- `--coverfile` requires `--embed` , can be `-`
- `--stegofile` requires either `--embed` or `--extract`, can be `-`
- `--nochecksum` requires `--embed`
- `--encryption` requires `--embed`, libmcrypt support?
- `--radius` requires `--embed`, must be unsigned long
- `--goal` requires `--embed`, what is a goal?
- `--marker` requires `-e-mbed` or `-extract`, "marker length" can only be used with `--embed`
- `--force` requires `--embed` or `--extract`
- `--debug` can be one of:
- - `--printgraph`
- - `--printgmlvertex` is a comma separated numbers, `<red_depth>,<start_vertex>`
- - `--debuglevel` is an unsigned int
- - `--check` requires `--embed`