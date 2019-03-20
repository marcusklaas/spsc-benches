#![feature(test)]

extern crate test;

mod channels {
    use crossbeam;

    const CHANNEL_BUFFER_SIZE: usize = 12;
    const BENCH_ITERS: usize = 10_000;

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

