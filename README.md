See https://discourse.julialang.org/t/trivial-port-from-c-rust-to-julia/105787

Comparison of Julia, C++, and Rust for an optimization problem.

I optimized the Julia version a bit with respect to the original version,
so it is significantly faster than the C++ and Rust versions.
The C++ and Rust versions are original, but could no doubt be optimized.


The Julia version can optionally use a standard RNG from the stock `Random` package.
But the original RNG is tested here, with the original seed, in order to verify that
the output is exactly the same as the Rust version. The C++ version apparently uses
a different seed.

#### Julia version

About 7.7s

```shell
shell> cd julia_trnbias
shell> julia -q  --optimize=3 -O3
```

```julia
julia> include("trnbias.jl"); # Usage error below is ok, we will run at repl
Usage: julia script.jl <which> <ncases> <trend> <nreps>
julia> @time optimize(0, 1000, 0.01, 200)
typeof(rng) = MarsagliaRng
Mean IS=0.04462564329047784  OOS=-0.0031496723033457423  Bias=0.047775315593823586
  7.693620 seconds (40 allocations: 11.656 KiB)
```

#### Rust version

About 9.5s

```shell
shell> cd rust_trnbias
shell> /usr/bin/time cargo run --release -q --bin trnbias -- 0 1000 0.01 200
Mean IS=0.04462564329047784  OOS=-0.0031496723033457423  Bias=0.047775315593823586
9.51user 0.00system 0:09.53elapsed 99%CPU (0avgtext+0avgdata 18928maxresident)k
0inputs+8outputs (0major+4053minor)pagefaults 0swaps
```

#### C++ version

About 11.2 s

```shell
shell> cd cpp_trnbias
shell> ./compile_cpp.sh
shell> /bin/time ./trnbias 0 1000 0.01 200
```

<!--  LocalWords:  RNG 7s cd julia trnbias O3 jl ok repl ncases nreps typeof 5s
<!--  LocalWords:  rng MarsagliaRng OOS 51user 00system 0avgtext 0avgdata cpp
<!--  LocalWords:  18928maxresident 0inputs 8outputs 0major 4053minor 0swaps
<!--  LocalWords:  pagefaults
 -->
