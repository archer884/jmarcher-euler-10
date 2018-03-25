extern crate primal;

use primal::Sieve;

trait Primality {
    fn is_prime(&self, n: u64) -> bool;
}

fn main() {
    let mut filter = NaivePrimality;
    let sum: u64 = (1..2_000_000).filter(|&n| filter.is_prime(n)).sum();
    println!("{}", sum);
}

struct NaivePrimality;

impl Primality for NaivePrimality {
    fn is_prime(&self, n: u64) -> bool {
        fn internal_is_prime(n: u64) -> bool {
            for i in 2..(n - 1) {
                if n % i == 0 {
                    return false;
                }
            }
            true
        }

        match n {
            1 => false,
            2 => true,
            n => internal_is_prime(n),
        }
    }
}

struct SievePrimality(Sieve);

impl SievePrimality {
    fn new(limit: u64) -> Self {
        SievePrimality(Sieve::new(limit as usize))
    }
}

impl Primality for SievePrimality {
    fn is_prime(&self, n: u64) -> bool {
        self.0.is_prime(n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naive_primality_works() {
        let mut naive = NaivePrimality;
        let mut sieve = SievePrimality::new(1000);

        for i in 1..=1000 {
            assert_eq!(
                naive.is_prime(i),
                sieve.is_prime(i),
                "Incorrect for: {}", i
            );
        }
    }
}
