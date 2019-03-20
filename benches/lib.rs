#![feature(test)]

extern crate test;

mod channels {
    use crossbeam;

    const CHANNEL_BUFFER_SIZE: usize = 12;
    const BENCH_ITERS: usize = 10_000;
    const VECTOR_CHUNK_SIZE: usize = 100;

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
    fn crossbeam_mpmc_vector_chunked(b: &mut test::Bencher) {
        b.iter(|| {
            crossbeam::scope(move |scope| {
                let (sender, receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);
                let mut iter = (0..BENCH_ITERS).into_iter();
                scope.spawn(move |_| {
                    let mut cont = true;
                    while cont {
                        let mut vek = Vec::with_capacity(VECTOR_CHUNK_SIZE);
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

                assert_eq!(BENCH_ITERS, receiver.into_iter().flatten().count());
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
                    total += vek.drain(..).count();
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
    fn crossbeam_mpmc(b: &mut test::Bencher) {
        b.iter(|| {
            crossbeam::scope(move |scope| {
                let (sender, receiver) = crossbeam::channel::bounded(CHANNEL_BUFFER_SIZE);
                scope.spawn(move |_| {
                    for i in 0..BENCH_ITERS {
                        sender.send(i).unwrap();
                    }            
                });

                assert_eq!(BENCH_ITERS, receiver.into_iter().count());
            });
        });
    }
}

