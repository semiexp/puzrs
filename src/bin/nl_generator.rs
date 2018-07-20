extern crate getopts;
extern crate puzrs;
extern crate rand;

use getopts::Matches;
use getopts::Options;
use std::env;

use puzrs::*;
use std::io;
use std::time::Instant;

use std::io::Write;
use std::sync::Mutex;
use std::thread;

macro_rules! run_timed {
    ($timer: ident, $flag: ident, $e: expr) => {
        if $flag {
            let start = Instant::now();
            let ret = $e;
            let end = start.elapsed();
            let cost_time = end.as_secs() as f64 + end.subsec_nanos() as f64 / 1e9f64;

            let mut timer_lock = $timer.lock().unwrap();
            *timer_lock += cost_time;

            ret
        } else {
            $e
        }
    };
}
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

#[derive(Clone, Copy, Debug)]
struct GeneratorOption {
    height: i32,
    width: i32,
    jobs: i32,
    no_adjacent_clues: bool,
    symmetry: Symmetry,
    minimum_path_length: i32,
    empty_width: i32,
    max_clue: Option<i32>,
    corner: Option<(i32, i32)>,
    use_profiler: bool,
    prioritized_extension: bool,
}

fn run_generator(opts: GeneratorOption) {
    let height = opts.height;
    let width = opts.width;
    let mut ths = vec![];
    let gen_probs = std::sync::Arc::new(Mutex::new(0i64));

    // profiling
    let use_profiler = opts.use_profiler;
    let cost_genenerator = std::sync::Arc::new(Mutex::new(0.0f64));
    let cost_pretest = std::sync::Arc::new(Mutex::new(0.0f64));
    let cost_exact_test = std::sync::Arc::new(Mutex::new(0.0f64));

    for _ in 0..opts.jobs {
        let gen_probs = gen_probs.clone();
        let cost_genenerator = cost_genenerator.clone();
        let cost_pretest = cost_pretest.clone();
        let cost_exact_test = cost_exact_test.clone();

        let opts = opts;

        ths.push(thread::spawn(move || {
            let start = Instant::now();

            let mut generator = numberlink::PlacementGenerator::new(height, width);

            let mut rng = rand::thread_rng();
            loop {
                let end = numberlink::generate_endpoint_constraint(
                    height,
                    width,
                    opts.empty_width,
                    opts.corner,
                    opts.symmetry,
                    &mut rng,
                );
                let opt = numberlink::GeneratorOption {
                    chain_threshold: opts.minimum_path_length,
                    endpoint_constraint: Some(&end),
                    forbid_adjacent_clue: opts.no_adjacent_clues,
                    symmetry: opts.symmetry,
                    clue_limit: opts.max_clue,
                    prioritized_extension: opts.prioritized_extension,
                };

                let placement = run_timed!(
                    cost_genenerator,
                    use_profiler,
                    generator.generate(&opt, &mut rng)
                );
                if let Some(placement) = placement {
                    // pretest
                    let pretest_res = run_timed!(
                        cost_pretest,
                        use_profiler,
                        numberlink::uniqueness_pretest(&placement)
                    );
                    if !pretest_res {
                        continue;
                    }

                    let problem = numberlink::extract_problem(&placement, &mut rng);

                    let ans = run_timed!(
                        cost_exact_test,
                        use_profiler,
                        numberlink::solve2(&problem, Some(2), false, true)
                    );

                    if ans.len() == 1 && !ans.found_not_fully_filled {
                        let stdin = io::stdout();
                        let handle = &mut stdin.lock();

                        let end = start.elapsed();
                        let cost_time =
                            (end.as_secs() as f64 + end.subsec_nanos() as f64 / 1e9f64) / 60f64;
                        let mut cnt = gen_probs.lock().unwrap();
                        *cnt += 1;
                        eprintln!(
                            "{} problem(s) in {:.3}[min] ({:.3} [prob/min])",
                            *cnt,
                            cost_time,
                            (*cnt) as f64 / cost_time
                        );
                        if use_profiler {
                            let cost_genenerator = *(cost_genenerator.lock().unwrap());
                            let cost_pretest = *(cost_pretest.lock().unwrap());
                            let cost_exact_test = *(cost_exact_test.lock().unwrap());
                            let cost_total = cost_genenerator + cost_pretest + cost_exact_test;

                            eprintln!("Generator: {:.3}[s] ({:.2}%) / Pretest: {:.3}[s] ({:.2}%) / Exact test: {:.3}[s] ({:.2}%)",
                                cost_genenerator, cost_genenerator / cost_total * 100.0f64,
                                cost_pretest, cost_pretest / cost_total * 100.0f64,
                                cost_exact_test, cost_exact_test / cost_total * 100.0f64);
                        }

                        writeln!(handle, "{} {}", height, width).unwrap();
                        for y in 0..height {
                            for x in 0..width {
                                let numberlink::Clue(c) = problem[(Y(y), X(x))];
                                if c >= 1 {
                                    write!(
                                        handle,
                                        "{}{}",
                                        c,
                                        if x == width - 1 { '\n' } else { ' ' }
                                    ).unwrap();
                                } else {
                                    write!(handle, ".{}", if x == width - 1 { '\n' } else { ' ' })
                                        .unwrap();
                                }
                            }
                        }
                        writeln!(handle).unwrap();
                    }
                }
            }
        }));
    }
    for th in ths {
        th.join().unwrap();
    }
}
fn parse_symmetry(s: String) -> Result<Symmetry, &'static str> {
    let mut ret = Symmetry::none();
    let tokens = s.split(',');
    for token in tokens {
        if token == "d" || token == "dyad" || token == "180" {
            ret.dyad = true;
        } else if token == "t" || token == "tetrad" || token == "90" {
            ret.tetrad = true;
        } else if token == "h" || token == "horizontal" {
            ret.horizontal = true;
        } else if token == "v" || token == "vertical" {
            ret.vertical = true;
        } else {
            return Err("Unrecognized symmetry option");
        }
    }
    return Ok(ret);
}
fn parse_options(matches: Matches) -> Result<GeneratorOption, &'static str> {
    let height = try!(
        matches
            .opt_str("h")
            .ok_or("'height' must be specified")
            .and_then(|arg| arg.parse::<i32>()
                .map_err(|_| "Could not parse value for 'height'"))
            .and_then(|arg| if arg > 0 {
                Ok(arg)
            } else {
                Err("'height' must be a positive integer")
            })
    );
    let width = try!(
        matches
            .opt_str("w")
            .ok_or("'width' must be specified")
            .and_then(|arg| arg.parse::<i32>()
                .map_err(|_| "Could not parse value for 'width'"))
            .and_then(|arg| if arg > 0 {
                Ok(arg)
            } else {
                Err("'width' must be a positive integer")
            })
    );
    let jobs = try!(
        matches
            .opt_str("j")
            .map(|s| s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'jobs'")
                .and_then(|arg| if arg > 0 {
                    Ok(arg)
                } else {
                    Err("'jobs' must be a positive integer")
                }))
            .unwrap_or(Ok(1))
    );
    let no_adjacent_clues = matches.opt_present("no-adjacent-clues");
    let symmetry = try!(
        matches
            .opt_str("s")
            .map(parse_symmetry)
            .unwrap_or(Ok(Symmetry::none()))
    );
    let minimum_path_length = try!(
        matches
            .opt_str("minimum-path-length")
            .map(|s| s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'minimum-path-length'")
                .and_then(|arg| if arg > 0 {
                    Ok(arg)
                } else {
                    Err("'minimum-path-length' must be a positive integer")
                }))
            .unwrap_or(Ok(1))
    );
    let empty_width = try!(
        matches
            .opt_str("empty-width")
            .map(|s| s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'empty-width'")
                .and_then(|arg| if arg > 0 {
                    Ok(arg)
                } else {
                    Err("'empty-width' must be a positive integer")
                }))
            .unwrap_or(Ok(1))
    );
    let max_clue = try!(
        matches
            .opt_str("max-clue")
            .map(|s| s.parse::<i32>()
                .map_err(|_| "Could not parse value for 'max-clue'")
                .and_then(|arg| if arg > 0 {
                    Ok(Some(arg))
                } else {
                    Err("'max-clue' must be a positive integer")
                }))
            .unwrap_or(Ok(None))
    );
    let use_profiler = matches.opt_present("use-profiler");
    let prioritized_extension = matches.opt_present("prioritized-extension");
    let corner = match matches.opt_str("corner") {
        Some(s) => {
            let split = s.split(",").collect::<Vec<&str>>();
            if split.len() != 2 {
                return Err("Could not parse value for 'corner'");
            }
            let lo = try!(
                split[0]
                    .parse::<i32>()
                    .map_err(|_| "Could not parse value for 'corner'")
            );
            let hi = try!(
                split[1]
                    .parse::<i32>()
                    .map_err(|_| "Could not parse value for 'corner'")
            );
            if !(1 <= lo && lo <= hi) {
                return Err("'corner' must be a valid range on positive integers");
            }
            Some((lo, hi))
        }
        None => None,
    };
    Ok(GeneratorOption {
        height,
        width,
        jobs,
        no_adjacent_clues,
        symmetry,
        minimum_path_length,
        empty_width,
        max_clue,
        corner,
        use_profiler,
        prioritized_extension,
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut options = Options::new();
    options.optflag("", "help", "Show this help menu");
    options.optopt("h", "height", "Height of desired problems", "10");
    options.optopt("w", "width", "Width of desired problems", "10");
    options.optopt("j", "jobs", "Number of workers (threads)", "2");
    options.optflag("a", "no-adjacent-clues", "Disallow adjacent clues");
    options.optopt("s", "symmetry", "Force symmetry", "180");
    options.optopt(
        "m",
        "minimum-path-length",
        "Minimum length of paths in the answer",
        "12",
    );
    options.optopt(
        "e",
        "empty-width",
        "Disallow clues on n cell(s) from the outer border",
        "1",
    );
    options.optopt(
        "c",
        "corner",
        "Put one clue within specified range from each corner",
        "1,3",
    );
    options.optopt("x", "max-clue", "Maximum value of clues", "10");
    options.optflag("p", "use-profiler", "Enable profiler");
    options.optflag(
        "r",
        "prioritized-extension",
        "Use prioritized extension in generator",
    );

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("error: {}", f);
            return;
        }
    };

    if matches.opt_present("help") {
        print_usage(&program, options);
        return;
    }

    let opt = match parse_options(matches) {
        Ok(opt) => opt,
        Err(f) => {
            eprintln!("error: {}", f);
            return;
        }
    };
    run_generator(opt);
}
