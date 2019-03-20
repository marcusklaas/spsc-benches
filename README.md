# spsc benchmarks

This is a small repository to benchmark several ways of moving lots of small items between
threads in Rust. Specifically, we test channels with a single producer (writer) and a single consumer.

Run all benchmarks by executing `cargo bench` on your commandline. That is all.

## Overview of implementations

- [Crossbeam]'s general purpose bounded multi-producer multi-consumer channel without chunking.
- Crossbeam's MPMC channel with chunking, which entails sending vectors of values instead of
  values themselves.
- Crossbeam's MPMC channel with chunking and recycling, where we use another channel to return
  empty vectors back to the producing channel for reuse.

## Results

The table below shows benchmarks results run on a fairly old AMD X4 860K. Modern processors should do
much better. However, these data should give an indication on how the implementations stack up to
eachother. All benchmarks but the overhead one measure the transfer of 10,000 word-sized integers from
between two threads (one-way). Other parameters are included in the table when applicable.

| implementation           | details                                  | time (nanoseconds)/ iter |
| ------------------------ | ---------------------------------------- | ------------------------ |
| overhead                 |                                          | 46,654                   |
| crossbeam mpmc           | channel buffer size = 12                 | 872,235                  |
| mpmc hunked              | channel buffer = 12, chunk size = 100    | 162,420                  |
| mpmc recycled chunks     |                                          | 100,580                  |

[crossbeam]: https://github.com/crossbeam-rs/crossbeam