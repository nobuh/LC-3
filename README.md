LC-3
====

LC-3 emulator in rust.

Usage
-----

- Load program in the hex (0x...) or the binary (0b...) text format.

    cat some_program.txt | cargo run

TODO
----

- [x] load bin format file.
- [ ] more I/O feature.
- [ ] complete cpu emulation for previledge and memory management.

Tips
----

Easily create binary file using xxd.

    echo "41 42 00 ff" | xxd -r -p > output.bin 
