You can run trnbias.jl like this.

#### At the Julia REPL

```shell
> julia -q  --optimize=3 -O3
```

```julia
julia> include("trnbias.jl")
julia> @time optimize(0, 1000, 0.01, 200)
```

You might repeat the last line to eliminate compile time. But the
compile time might be less than fluctuations in the run time.


#### At the cli, including compilation


```shell
/usr/bin/time julia --optimize=3 -O3 trnbias.jl 0 1000 0.01 200
```

The latter adds a few hundred milliseconds in compile time
