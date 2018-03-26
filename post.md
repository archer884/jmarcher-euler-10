Let's dig into the [latest challenge](https://jmarcher.io/programming-challenge-euler-10/) with an eye toward why it may be challenging for new programmers. This is a complicated story and it'll require us to dig into something that I'm really not formally trained in: asymptotic complexity. Luckily, a good buddy of mine is pretty well-versed in this stuff, although it's admittedly been about a century since he got his CS degree.

Let's start by defining an interface that all our prime testing mechanisms will conform to:

```rust
trait Primality {
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

```shell
[master ≡ +0 ~1 -0 !]> Measure-Command { .'.\target\release\jmarcher-euler-10.exe' }

Days              : 0
Hours             : 0
Minutes           : 14
Seconds           : 44
Milliseconds      : 616
Ticks             : 8846168551
TotalDays         : 0.0102386210081019
TotalHours        : 0.245726904194444
TotalMinutes      : 14.7436142516667
TotalSeconds      : 884.6168551
TotalMilliseconds : 884616.8551
```

...For those of you who have trouble reading PowerShell, that's a runtime of fourteen minutes, forty-four seconds.

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

**Note:** I'm using inclusive ranges, which are stable-ish as of `rustc 1.26.0-nightly (f5631d9ac 2018-03-24)`. If you're on the stable branch or have an older version, you'll need to make an adjustment there.

This (along with a lot of the other optimizations you could make) is a linear reduction in the problem space. Testing only odd odd candidates, testing candidates using only odd numbers; both of these are, likewise, linear reductions which do not factor into asymptotic complexity. The reason is that, for some (perhaps astronomically large? But probably not) value of N, you're still going to hit a wall, because the cost associated with N increases at a greater-than-linear rate. At the end of the day, *half of infinity is still infinity.*

As expected, the change above results in a reduction of our search space, and therefore our search time, by half:

```shell
[master ≡ +0 ~2 -0 !]> Measure-Command { .'.\target\release\jmarcher-euler-10.exe' }

Days              : 0
Hours             : 0
Minutes           : 7
Seconds           : 42
Milliseconds      : 691
Ticks             : 4626918700
TotalDays         : 0.00535522997685185
TotalHours        : 0.128525519444444
TotalMinutes      : 7.71153116666667
TotalSeconds      : 462.69187
TotalMilliseconds : 462691.87
```

...Seven minutes, forty-two seconds.

Now, there are a few other little time-savers we can capture similarly. For instance, a reduction in the number of tests should improve our time by half again, and we can actually divide by three instead of two to further reduce, again linearly, our runtime. Let's apply both of those.

## Pulling out all the stops (linearly)

One thing we aren't allowed to do here is a linear reduction in the number of candidates, because the interface requires that all candidates from one to two million (exclusive) be tested. However, everything else I've talked about is implemented:

```rust
struct FastLinearPrimality;

impl Primality for FastLinearPrimality {
    fn is_prime(&self, n: u64) -> bool {
        struct TestRange {
            current: u64,
            limit: u64,
        }

        impl TestRange {
            fn new(limit: u64) -> TestRange {
                TestRange { current: 3, limit }
            }
        }

        impl Iterator for TestRange {
            type Item = u64;

            fn next(&mut self) -> Option<Self::Item> {
                match self.current {
                    ret if ret > self.limit => None,
                    ret => {
                        self.current += 2;
                        Some(ret)
                    }
                }
            }
        }

        fn internal_is_prime(n: u64) -> bool {
            let max = n / 3;
            for i in TestRange::new(max) {
                if n % i == 0 {
                    return false;
                }
            }
            true
        }

        match n {
            1 => false,
            2 | 3 => true,
            n if n & 1 == 0 => false, // No even numbers
            n => internal_is_prime(n),
        }
    }
}
```

As you can see, this implementation required a custom iterator because, even in nightly rust, the `.step_by(foo)` iterator adapter isn't a thing yet. Runtime for this version represents another linear reduction, exactly as expected:

```shell
[master ≡ +0 ~2 -0 !]> Measure-Command { .'.\target\release\jmarcher-euler-10.exe' }

Days              : 0
Hours             : 0
Minutes           : 2
Seconds           : 27
Milliseconds      : 564
Ticks             : 1475649548
TotalDays         : 0.00170792771759259
TotalHours        : 0.0409902652222222
TotalMinutes      : 2.45941591333333
TotalSeconds      : 147.5649548
TotalMilliseconds : 147564.9548
```

...Two minutes, thirty seconds. Or, in other words, "fast linear primality" is kind of a misnomer; this is still slow as next year's Christmas. So, why the hell doesn't this work? to answer that question, let's characterize the problem space itself.

# What is the problem, exactly?

The search space for the problem at hand ("Is N prime?") is equivalent to pretty much all the numbers less than N. We can draw it like this:

[`y=x`](https://www.desmos.com/calculator/fzz0jcm5ur)

...where *y* is equal to the number of tests required and *x* is our candidate.

The linear reduction applied above makes the graph look like this instead:

[`y=x/3`](https://www.desmos.com/calculator/hesauvjfdk)

Now, there's no denying that's a significant improvement. Even so, by the time *x* reaches 300,000, *y* has hit a hundred grand. A hundred grand is a big number of things when you're going to be doing them **1.7 million times.** (That's a hundred and seventy billion tests, by the way--and this is just a napkin-math-style lower bound.)

The point is that as these numbers get bigger, so does the search space, and these numbers (up to two million, remember?) get *very* big, at least in human terms. In actual fact, two million isn't exactly "big" as far as a computer is concerned. Want proof? Here's the runtime for the sieve version:

```shell
[master ≡ +0 ~2 -0 !]> Measure-Command { .'.\target\release\jmarcher-euler-10.exe' }

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 0
Milliseconds      : 134
Ticks             : 1346466
TotalDays         : 1.55840972222222E-06
TotalHours        : 3.74018333333333E-05
TotalMinutes      : 0.00224411
TotalSeconds      : 0.1346466
TotalMilliseconds : 134.6466
```

134 milliseconds. Note that a runtime of 134 milliseconds makes this program fast enough that the shell is no longer able to time it accurately; a time in the range of one hundred milliseconds is about the *least* time the shell is able to measure. This applies to macOS as well.

So, point is that if we use the computer's resources efficiently, this problem is *easily* solvable, because the problem is not in fact very big. But how can we be more intelligent? Specifically, how can we be more intelligent *without* resorting to a sieve? Because, as I'm concerned, that's just not a realistic option for someone still learning the basics. The answer is that we have to make some kind of *non-linear* improvement.

## A non-linear reduction in problem space

Here's another graph:

[`y=sqrt(x)`](https://www.desmos.com/calculator/1nbp5vtuw1)

It may not be obvious at first, but there *is* a line on this graph; it's just that, for most values of *x,* *y* is all but indistinguishable from the baseline of zero. Anyway, here's the logic: the greatest factor we need to search in order to determine the primality of any number is that number's square root, because once we go past that, we *know* we've already tested the other, smaller factor. There's no need to check to see whether 7 evenly divides 21 because we already know that 3 evenly divides 21. If we get to the square root of N and N is *not* found to be a perfect square, N is guaranteed to be prime.

So what's that look like when we implement it?

```rust
struct NaivePrimalityWithNonLinearRangeLimit;

impl Primality for NaivePrimalityWithNonLinearRangeLimit {
    fn is_prime(&self, n: u64) -> bool {
        fn internal_is_prime(n: u64) -> bool {
            let max = (n as f64).sqrt() as u64 + 1;
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

Note that I've not included any other optimizations. Also note that the name for that unit struct is getting awfully long. I might have to change it to Sally or something.

```
[master ≡ +0 ~2 -0 !]> Measure-Command { .'.\target\release\jmarcher-euler-10.exe' }

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 1
Milliseconds      : 247
Ticks             : 12473922
TotalDays         : 1.44374097222222E-05
TotalHours        : 0.000346497833333333
TotalMinutes      : 0.02078987
TotalSeconds      : 1.2473922
TotalMilliseconds : 1247.3922
```

*1.25 seconds.* From a quarter of an hour to less than two seconds based on only that change in the range limit. We can try out the other optimizations as well, but do they even matter at this point? In point of fact, the time drops to 725 milliseconds, which is an improvement, but, of course, a linear improvement. Who cares about those?

Let's talk about why this whole thing goes off the rails so fast as soon as we introduce that `sqrt()` function.

# O(dammit are you serious?)

<!-- Ask David for assistance on getting the O(bullshit) right-ish. -->
