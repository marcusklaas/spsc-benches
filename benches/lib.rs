#![feature(test)]

extern crate test;

mod channels {
    use crossbeam;
    use spsc_benches::{produce_value, consume_value};

    const CHANNEL_BUFFER_SIZE: usize = 12;
    const BENCH_ITERS: usize = 1000;
    const VECTOR_CHUNK_SIZE: usize = 100;
    const PRODUCTION_DIFFICULTY: usize = 100;
    const CONSUMPTION_DIFFICULTY: usize = 100;

    #[bench]
    fn overhead(b: &mut test::Bencher) {
        b.iter(|| {
            crossbeam::scope(move |scope| {
                let (sender, receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);
                scope.spawn(move |_| {
                    sender.send(0).unwrap();        
                });

                assert_eq!(1, receiver.into_iter().count());
            });
        });
    }

    #[bench]
    fn sequential(b: &mut test::Bencher) {
        b.iter(|| {
            for _ in 0..BENCH_ITERS {
                produce_value(PRODUCTION_DIFFICULTY);
                consume_value(PRODUCTION_DIFFICULTY);
            }
        });
    }

    #[bench]
    fn crossbeam_mpmc_vector_chunked(b: &mut test::Bencher) {
        b.iter(|| {
            crossbeam::scope(move |scope| {
                let (sender, receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);
                let mut iter = (0..BENCH_ITERS).into_iter();
                scope.spawn(move |_| {
                    let mut cont = true;
                    while cont {
                        let mut vek = Vec::with_capacity(VECTOR_CHUNK_SIZE);
                        while let Some(i) = iter.next() {
                            vek.push(produce_value(PRODUCTION_DIFFICULTY));
                            if vek.len() == VECTOR_CHUNK_SIZE {
                                break;
                            }                    
                        }
                        cont = vek.len() == VECTOR_CHUNK_SIZE;
                        sender.send(vek).unwrap();
                    }        
                });

                let mut total = 0;
                for i in receiver.into_iter().flatten() {
                    consume_value(CONSUMPTION_DIFFICULTY);
                    total += 1;
                }
                assert_eq!(BENCH_ITERS, total);
            });
        });
    }

    #[bench]
    fn crossbeam_mpmc_vector_chunked_recycling(b: &mut test::Bencher) {
        b.iter(|| {
            crossbeam::scope(move |scope| {
                let (sender, receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);
                let (recycle_sender, recycle_receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);

                let mut iter = (0..BENCH_ITERS).into_iter();
                scope.spawn(move |_| {
                    let mut cont = true;
                    while cont {
                        let mut vek = recycle_receiver
                            .try_recv()
                            .unwrap_or_else(|_| Vec::with_capacity(VECTOR_CHUNK_SIZE));

                        while let Some(event) = iter.next() {
                            vek.push(event);
                            if vek.len() == VECTOR_CHUNK_SIZE {
                                break;
                            }                    
                        }
                        cont = vek.len() == VECTOR_CHUNK_SIZE;
                        sender.send(vek).unwrap();
                    }        
                });

                let mut total = 0;
                let mut recycling = true;
                for mut vek in receiver {
                    for i in vek.drain(..) {
                        consume_value(CONSUMPTION_DIFFICULTY);
                        total += 1;
                    }
                    if recycling {
                        // stop trying to recycle if other side has hung up
                        recycling = recycle_sender.send(vek).is_ok();
                    }
                }

                assert_eq!(BENCH_ITERS, total);
            });
        });
    }

    #[bench]
    fn crossbeam_mpmc_simple(b: &mut test::Bencher) {
        b.iter(|| {
            crossbeam::scope(move |scope| {
                let (sender, receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);
                scope.spawn(move |_| {
                    for i in 0..BENCH_ITERS {
                        sender.send(produce_value(PRODUCTION_DIFFICULTY)).unwrap();
                    }            
                });

                assert_eq!(BENCH_ITERS, receiver.into_iter().map(|i| consume_value(CONSUMPTION_DIFFICULTY)).count());
            });
        });
    }
}

