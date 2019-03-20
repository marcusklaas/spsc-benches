pub fn produce_value(difficulty: usize) -> usize {
    (1..difficulty).into_iter().fold(1, |x, y| x * y)
}

pub fn consume_value(difficulty: usize) -> usize {
    (1..difficulty).into_iter().fold(1, |x, y| x * y)
}

