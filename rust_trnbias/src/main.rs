use std::{num::Wrapping, time::SystemTime};

use rand::distributions::Distribution;
use rand::{RngCore, SeedableRng};

// This is a random int generator suggested by Marsaglia in his DIEHARD suite.
// It provides a great combination of speed and quality.
// We also have unifrand(), a random 0-1 generator based on it.
pub struct MarsagliaRng {
    q: [u32; 256],
    carry: u32,
    mwc256_initialized: bool,
    mwc256_seed: i32,
    i: Wrapping<u8>,
}
#[derive(Default)]
pub struct MarsagliaRngDist {}

impl Distribution<f64> for MarsagliaRngDist {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        let mult: f64 = 1.0 / (0xFFFFFFFF as u64) as f64;
        mult * rng.next_u32() as f64
    }
}

impl Default for MarsagliaRng {
    fn default() -> Self {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let ts_nano = duration_since_epoch.as_nanos(); // u128
        MarsagliaRng::from_seed([
            ts_nano as u8,
            (ts_nano >> 8) as u8,
            (ts_nano >> 16) as u8,
            (ts_nano >> 24) as u8,
        ])
    }
}

impl RngCore for MarsagliaRng {
    fn next_u32(&mut self) -> u32 {
        let a: u64 = 809430660;

        if !self.mwc256_initialized {
            let mut j = Wrapping(self.mwc256_seed as u32);
            let c1 = Wrapping(69069);
            let c2 = Wrapping(12345);
            self.mwc256_initialized = true;
            for k in 0..256 {
                j = c1 * j + c2;
                self.q[k] = j.0;
            }
        }

        self.i += 1;
        let t = a * self.q[self.i.0 as usize] as u64 + self.carry as u64;
        self.carry = (t >> 32) as u32;
        self.q[self.i.0 as usize] = (t & 0xFFFFFFFF) as u32;

        self.q[self.i.0 as usize]
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u32() as u64
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        todo!()
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        todo!()
    }
}

impl SeedableRng for MarsagliaRng {
    type Seed = [u8; 4];

    fn from_seed(seed: Self::Seed) -> Self {
        MarsagliaRng {
            q: [0; 256],
            carry: 362436,
            mwc256_initialized: false,
            mwc256_seed: i32::from_le_bytes(seed),
            i: Wrapping(255u8),
        }
    }
}

struct ParamsResult {
    short_term: usize,
    long_term: usize,
    performance: f64,
}

fn opt_params(which: usize, ncases: usize, x: &[f64]) -> ParamsResult {
    let mut best_perf = f64::MIN;
    let mut ibestshort = 0;
    let mut ibestlong = 0;

    for ilong in 2..200 {
        for ishort in 1..ilong {
            let mut total_return = 0.0;
            let mut win_sum = 1e-60;
            let mut lose_sum = 1e-60;
            let mut sum_squares = 1e-60;
            let (mut short_sum, mut long_sum) = (0.0, 0.0);

            for i in (ilong - 1)..(ncases - 1) {
                if i == ilong - 1 {
                    short_sum = (i + 1 - ishort..=i).map(|j| x[j]).sum();
                    long_sum = (i + 1 - ilong..=i).map(|j| x[j]).sum();
                } else {
                    short_sum += x[i] - x[i - ishort];
                    long_sum += x[i] - x[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                let ret = if short_mean > long_mean {
                    x[i + 1] - x[i]
                } else if short_mean < long_mean {
                    x[i] - x[i + 1]
                } else {
                    0.0
                };

                total_return += ret;
                sum_squares += ret * ret;
                if ret > 0.0 {
                    win_sum += ret;
                } else {
                    lose_sum -= ret;
                }
            }

            match which {
                0 => {
                    total_return /= (ncases - ilong) as f64;
                    if total_return > best_perf {
                        best_perf = total_return;
                        ibestshort = ishort;
                        ibestlong = ilong;
                    }
                }
                1 => {
                    let pf = win_sum / lose_sum;
                    if pf > best_perf {
                        best_perf = pf;
                        ibestshort = ishort;
                        ibestlong = ilong;
                    }
                }
                2 => {
                    total_return /= (ncases - ilong) as f64;
                    sum_squares /= (ncases - ilong) as f64;
                    sum_squares -= total_return * total_return;
                    let sr = total_return / (sum_squares.sqrt() + 1e-8);
                    if sr > best_perf {
                        best_perf = sr;
                        ibestshort = ishort;
                        ibestlong = ilong;
                    }
                }
                _ => {}
            }
        }
    }

    ParamsResult {
        performance: best_perf,
        short_term: ibestshort,
        long_term: ibestlong,
    }
}

fn test_system(ncases: usize, x: &[f64], short_term: usize, long_term: usize) -> f64 {
    let mut sum = 0.0;

    for i in (long_term - 1)..(ncases - 1) {
        let short_mean = (i + 1 - short_term..=i).map(|j| x[j]).sum::<f64>() / short_term as f64;
        let long_mean = (i + 1 - long_term..=i).map(|j| x[j]).sum::<f64>() / long_term as f64;

        if short_mean > long_mean {
            sum += x[i + 1] - x[i];
        } else if short_mean < long_mean {
            sum -= x[i + 1] - x[i];
        }
    }

    sum / (ncases - long_term) as f64
}

fn main() {
    let which = std::env::args()
        .nth(1)
        .expect("which metric to optimize - 0=mean return  1=profit factor  2=Sharpe ratio")
        .parse::<usize>()
        .unwrap();

    let ncases = std::env::args()
        .nth(2)
        .expect("ncases - number of training and test cases")
        .parse::<usize>()
        .unwrap();

    let save_trend = std::env::args()
        .nth(3)
        .expect("trend - Amount of trending, 0 for flat system, 0.02 small trend, 0.2 large trend")
        .parse::<f64>()
        .unwrap();

    let nreps = std::env::args()
        .nth(4)
        .expect("nreps - number of test replications - at least a few thousand for statistical significance")
        .parse::<usize>()
        .unwrap();

    let mut x = vec![0.0; ncases];

    let mut is_mean = 0.0;
    let mut oos_mean = 0.0;

    let mut rnd = MarsagliaRng::from_seed(33i32.to_le_bytes());
    let dist = MarsagliaRngDist::default();

    for _irep in 0..nreps {
        //generate in sample
        let mut trend = save_trend;
        x[0] = 0.0;
        for i in 1..ncases {
            if i % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1] + trend + dist.sample(&mut rnd) + dist.sample(&mut rnd)
                - dist.sample(&mut rnd)
                - dist.sample(&mut rnd);
        }

        let ParamsResult {
            short_term,
            long_term,
            performance: is_perf,
        } = opt_params(which, ncases, &x);

        // genearte oos
        trend = save_trend;
        x[0] = 0.0;
        for i in 1..ncases {
            if i % 50 == 0 {
                trend = -trend;
            }
            x[i] = x[i - 1] + trend + dist.sample(&mut rnd) + dist.sample(&mut rnd)
                - dist.sample(&mut rnd)
                - dist.sample(&mut rnd);
        }

        let oos_perf = test_system(ncases, &x, short_term, long_term);

        is_mean += is_perf;
        oos_mean += oos_perf;

        // println!(
        //     "{:3}: {:3} {:3}  {:8.4} {:8.4} ({:8.4})",
        //     irep,
        //     short_term,
        //     long_term,
        //     is_perf,
        //     oos_perf,
        //     is_perf - oos_perf
        // );
    }

    is_mean /= nreps as f64;
    oos_mean /= nreps as f64;

    println!(
        "Mean IS={}  OOS={}  Bias={}",
        is_mean,
        oos_mean,
        is_mean - oos_mean
    );
}
