# spsc benchmarks

This is a small repository to benchmark several ways of moving lots of small items between
threads in Rust. Specifically, we test channels with a single producer (writer) and a single consumer.

## Overview of implementations

- [Crossbeam]'s general purpose bounded multi-producer multi-consumer channel without chunking.

## Results

TODO

[crossbeam]: https://github.com/crossbeam-rs/crossbeam