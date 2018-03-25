Let's dig into the [latest challenge](https://jmarcher.io/programming-challenge-euler-10/) with an eye toward why it may be challenging for new programmers. This is a complicated story and it'll require us to dig into something that I'm really not formally trained in: asymptotic complexity. Luckily, a good buddy of mine is pretty well-versed in this stuff, although it's admittedly been about a century since he got his CS degree.

Let's start by defining an interface that all our prime testing mechanisms will conform to:

```rust
trait Primality {
    // Most of our implementations will not require a unique reference to self,
    // but the interface will allow for that just in case one does.
    fn is_prime(&self, n: u64) -> bool;
}
```

This'll ensure that the big difference between each implementation is just the actual primality test rather than any of the code wrapped around it. Instinct tells me there's a good chance this will actually harm the efficiency of the most efficient method (a sieve), but I'm actually all right with that, because I don't think it's going to make any real difference. Now that's settled, here's our first implementation:

```rust
struct NaivePrimality;

impl Primality for NaivePrimality {
    fn is_prime(&self, n: u64) -> bool {
        fn internal_is_prime(n: u64) -> bool {
            for i in 2..n {
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
```

As you can see, we deal with the usual annoyances (special cases for one and two) with a match expression and then forward all other calculations to an internal implementation for is_prime. I tested this by comparing results for this function vs. results for a sieve from the library `primal` for integers through 1000. As expected, the results are identical and the test completes instantaneously, even though tests are compiled in debug mode. Here's the implementation for the sieve:

```rust
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
```

I'm not clear why primal makes use of a `usize`; I imagine that it has to do with the underlying representation of a sieve, but it's not like I've ever looked at the code. Without further ado, let's find out what kind of performance we get out of our naive implementation.

# Naive primality check

The code for our solution (aside from the interfaces defined above) is dirt simple and, I think, obviously correct.

```rust
    let filter = NaivePrimality;
    let sum: u64 = (1..2_000_000)
        .filter(|&n| filter.is_prime(n))
        .sum();
        
    println!("{}", sum);
```

That said, this code ain't exactly swift. I started running this at the coffee shop where I was originally writing this article and quickly realized my laptop was going to run out of juice before the program actually finished. The runtime listed here is for my desktop, which has some fairly beastly hardware—and is unlikely to run out of juice before finishing something like this.

<!-- runtime goes here -->

Obviously, something has to be done to improve this situation. The challeng requires that this program complete in about the amount of time it takes Duke to sell the secret formula for the Bush family's world-famous baked beans, remember?

# Naive primality check with linear range limit

Almost everyone tries some variation on this. It's pretty obvious that it's an improvement, and it's also a pretty sure bet that it will work. Back when I was first learning C#, I thought to myself, "Self, there's no way that the square root of a given number is ever going to be more than half of that number." This code is similar to what I came up with back then:

```rust
struct NaivePrimalityWithRangeLimit;

impl Primality for NaivePrimalityWithRangeLimit {
    fn is_prime(&self, n: u64) -> bool {
        fn internal_is_prime(n: u64) -> bool {
            let max = n / 2;
            for i in 2..=max {
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
```

> Note: I'm using inclusive ranges, which are stable-ish as of `rustc 1.26.0-nightly (f5631d9ac 2018-03-24)`. If you're on the stable branch or have an older version, you'll need to make an adjustment there.
