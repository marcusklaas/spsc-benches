# spsc benchmarks

This is a small repository to benchmark several ways of moving lots of small items between
threads in Rust. Specifically, we test channels with a single producer (writer) and a single consumer.

Run all benchmarks by executing `cargo bench` on your commandline. That is all.

## Overview of implementations

- [Crossbeam]'s general purpose bounded multi-producer multi-consumer channel without chunking.

## Results

The table below shows benchmarks results run on a fairly old AMD X4 860K. Modern processors should do
much better. However, these data should give an indication on how the implementations stack up to
eachother. All benchmarks but the overhead one measure the transfer of 10,000 word-sized integers from
between two threads (one-way). Other parameters are included in the table when applicable.

| implementation                                           | time (nanoseconds)/ iter |
| -------------------------------------------------------- | ------------------------ |
| overhead                                                 | 46,654                   |
| crossbeam_mpmc                                           | 872,235                  |
| crossbeam_mpmc_vector_chunked                            | 162,420                  |
| crossbeam_mpmc_vector_chunked_recycling                  | 100,580                  |

[crossbeam]: https://github.com/crossbeam-rs/crossbeam