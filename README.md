# spsc benchmarks

This is a small repository to benchmark several ways of moving lots of small items between
threads in Rust. Specifically, we test channels with a single producer (writer) and a single consumer.

Suppose we have a workload that involves the creation and consumption of many elements. Instead
of doing the creation and consumption sequentially on the same thread, we could split the workload
between two threads, performing the creatiom and the consumption simultaneously. Theoretically,
we could increase the throughput of our process by 100%. However, starting threads, setting up
channels and transfer of items takes time itself, which is what we call overhead.

This repository aims to measure the effects on total processing times of different implementations
of such channels. The goal is to get as close to that 100% speedup as possible.

Run all benchmarks by executing `cargo bench` on your commandline.

## Overview of implementations

- Sequential implementation with production and consumption happening in the same thread. This
  is the baseline to beat.
- [Crossbeam]'s general purpose bounded multi-producer multi-consumer channel without chunking.
- Crossbeam's MPMC channel with chunking, which entails sending vectors of values instead of
  values themselves.
- Crossbeam's MPMC channel with chunking and recycling, where we use another channel to return
  empty vectors back to the producing channel for reuse.
- A custom single producer single consumer channel.

## Results

The table below shows benchmarks results run on a fairly old AMD X4 860K. Modern processors should do
much better. However, these data should give an indication on how the implementations stack up to
eachother. All benchmarks but the overhead one measure the production, consumption and transfer of
1000 word-sized integers from between two threads (one-way). Other parameters are included in the
table when applicable.

| implementation           | details                                  | time (nanoseconds)/ iter |
| ------------------------ | ---------------------------------------- | ------------------------ |
| overhead                 |                                          | 31,268                   |
| sequential               |                                          | 158,031                  |
| crossbeam mpmc           | channel buffer size = 16                 | 176,841                  |
| mpmc chunked             | channel buffer = 16, chunk size = 100    | 121,835                  |
| mpmc recycled chunks     |                                          | 116,025                  |
| custom spsc              | channel buffer size = 16                 | 118,067                  |

[crossbeam]: https://github.com/crossbeam-rs/crossbeam