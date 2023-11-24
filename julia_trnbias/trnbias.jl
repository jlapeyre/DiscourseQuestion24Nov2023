using Random
using Printf

struct ParamsResult
    short_term::Int
    long_term::Int
    performance::Float64
end

mutable struct MarsagliaRng
    q::Vector{UInt32}
    carry::UInt32
    mwc256_initialized::Bool
    mwc256_seed::Int32
    i::UInt8

    function MarsagliaRng(seed::Vector{UInt8})
        q = zeros(UInt32, 256)
        carry = 362436
        mwc256_initialized = false
        mwc256_seed = reinterpret(Int32, seed)[1]
        i = 255
        new(q, carry, mwc256_initialized, mwc256_seed, i)
    end
end

# Default constructor using system time for seeding
function MarsagliaRng()
    ts_nano = UInt8(nanosecond(now()))
    seed = [ts_nano, ts_nano >>> 8, ts_nano >>> 16, ts_nano >>> 24]
    MarsagliaRng(seed)
end

# Random number generator functions
function next_u32(rng::MarsagliaRng)::UInt32
    a = UInt64(809430660)

    if !rng.mwc256_initialized
        j = UInt32(rng.mwc256_seed)
        c1 = UInt32(69069)
        c2 = UInt32(12345)
        rng.mwc256_initialized = true
        for k in 1:256
            j = (c1 * j + c2) % UInt32
            rng.q[k] = j
        end
    end

    rng.i = (rng.i + 1) % 256
    t = a * UInt64(rng.q[rng.i+1]) + UInt64(rng.carry)
    rng.carry = (t >> 32)
    rng.q[rng.i+1] = t % UInt32

    return rng.q[rng.i+1]
end

function next_u64(rng::MarsagliaRng)::UInt64
    UInt64(next_u32(rng))
end

# EDIT: extend Base.rand rather than shadowing it.
# Sample function for generating a Float64
function Base.rand(rng::MarsagliaRng)::Float64
    mult = 1.0 / typemax(UInt32)
    mult * next_u32(rng)
end

function opt_params(which::Integer, ncases::Integer, x::Vector{Float64})
    FloatT = eltype(x)
    best_perf = typemin(FloatT)
    ibestshort = 0
    ibestlong = 0

    small_float = 1e-60
    @inbounds for ilong in 2:199
        for ishort in 1:(ilong-1)
            total_return = zero(FloatT)
            win_sum = small_float
            lose_sum = small_float
            sum_squares = small_float
            short_sum = zero(FloatT)
            long_sum = zero(FloatT)

            for i in (ilong-1:ncases-2)
                if i == ilong-1
                    short_sum = sum(@view x[(i-ishort+2):i+1])
                    long_sum = sum(@view x[(i-ilong+2):i+1])
                else
                    short_sum += x[i+1] - x[i-ishort+1]
                    long_sum += x[i+1] - x[i-ilong+1]
                end


                short_mean = short_sum / ishort
                long_mean = long_sum / ilong

                ret = (short_mean > long_mean) ? x[i+2] - x[i+1] :
                      (short_mean < long_mean) ? x[i+1] - x[i+2] : zero(FloatT)

                total_return += ret
                sum_squares += ret^2
                if ret > zero(FloatT)
                    win_sum += ret
                else
                    lose_sum -= ret
                end
            end

            if which == 0
                total_return /= (ncases - ilong)
                if total_return > best_perf
                    best_perf = total_return
                    ibestshort = ishort
                    ibestlong = ilong
                end
            elseif which == 1
                pf = win_sum / lose_sum
                if pf > best_perf
                    best_perf = pf
                    ibestshort = ishort
                    ibestlong = ilong
                end
            elseif which == 2
                total_return /= (ncases - ilong)
                sum_squares /= (ncases - ilong)
                sum_squares -= total_return^2
                sr = total_return / (sqrt(sum_squares) + 1e-8)
                if sr > best_perf
                    best_perf = sr
                    ibestshort = ishort
                    ibestlong = ilong
                end
            end
        end
    end

    ParamsResult(ibestshort, ibestlong, best_perf)
end

function test_system(ncases::Integer, x::Vector{Float64}, short_term::Integer, long_term::Integer)
    FloatT = eltype(x)
    sum1 = zero(FloatT)

    for i in (long_term-1:ncases-2)
        short_mean = sum(@view x[i-short_term+2:i+1]) / short_term
        long_mean = sum(@view x[i-long_term+2:i+1]) / long_term

        if short_mean > long_mean
            sum1 += x[i+2] - x[i+1]
        elseif short_mean < long_mean
            sum1 -= x[i+2] - x[i+1]
        end

    end
    sum1 / (ncases - long_term)
end

function main()
    if length(ARGS) < 4
        println("Usage: julia script.jl <which> <ncases> <trend> <nreps>")
        return
    end

    which = parse(Int, ARGS[1])
    ncases = parse(Int, ARGS[2])
    save_trend = parse(Float64, ARGS[3])
    nreps = parse(Int, ARGS[4])

    optimize(which, ncases, save_trend, nreps)
end

function optimize(which::Integer, ncases::Integer, save_trend::Float64, nreps::Integer, rng=MarsagliaRng(UInt8.([33, 0, 0, 0])))
    @show typeof(rng)
    _optimize(rng, which, ncases, save_trend, nreps)
end

function _optimize(rng, which::Integer, ncases::Integer, save_trend::Float64, nreps::Integer)
    print_results = false

    FloatT = typeof(save_trend)
    x = zeros(FloatT, ncases)

    is_mean = zero(FloatT)
    oos_mean = zero(FloatT)

    @inbounds for irep in 1:nreps
        # Generate in-sample
        trend = save_trend
        x[1] = zero(FloatT)
        for i in 2:ncases
            if (i - 1) % 50 == 0
                trend = -trend
            end
            x[i] = x[i-1] + trend + rand(rng) + rand(rng) - rand(rng) - rand(rng)
        end

        params_result = opt_params(which, ncases, x)

        # Generate out-of-sample
        trend = save_trend
        x[1] = zero(FloatT)
        for i in 2:ncases
            if (i - 1) % 50 == 0
                trend = -trend
            end
            x[i] = x[i-1] + trend + rand(rng) + rand(rng) - rand(rng) - rand(rng)
        end


        oos_perf = test_system(ncases, x, params_result.short_term, params_result.long_term)

        is_mean += params_result.performance
        oos_mean += oos_perf
        if print_results
            @printf("%3d: %3d %3d  %8.4f %8.4f (%8.4f)\n",
                    irep, params_result.short_term, params_result.long_term, params_result.performance, oos_perf, params_result.performance - oos_perf)
        end

    end

    is_mean /= nreps
    oos_mean /= nreps

    println("Mean IS=$is_mean  OOS=$oos_mean  Bias=$(is_mean - oos_mean)")

end

main()
