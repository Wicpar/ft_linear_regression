use std::env;
use ft_linear_regression::theta::{ThetaFileArg, save_theta};
use ft_linear_regression::args::{F64Parser, ArgParser, DefaultArgParser, arg_err};
use ft_linear_regression::dataset::{DatasetArg, Dataset, DatasetEntry};
use ft_linear_regression::estimate_price::estimate_price;
use std::time::Instant;


pub struct LearnRatioArg;

impl F64Parser<'_> for LearnRatioArg {
    const NAMES: &'static [&'static str] = &["-r", "--ratio"];
    const DESCRIPTION: &'static str = "The Learning Ratio";
}

impl DefaultArgParser<'_, f64> for LearnRatioArg {
    const DEFAULT: f64 = 0.00001;
}

fn main() {
    let args: Vec<_> = env::args().skip(1).map(|it| it.to_lowercase()).collect();
    let mut used = vec![false; args.len()];
    let theta_path = ThetaFileArg::try_parse(&args, &mut used);
    let dataset_path = DatasetArg::try_parse(&args, &mut used);
    let ratio = LearnRatioArg::parse(&args, &mut used);

    for (idx, it) in args.iter().enumerate() {
        if !used[idx] {
            arg_err(idx, it, "Arg is not recognized, ignoring. --help for more info");
        }
    }

    if let Ok(raw) = Dataset::read_from(dataset_path, None) {
        let mut theta = (0.0, 0.0);

        let normalized = raw.clone().normalize();

        let base = ratio / normalized.entries.len() as f64;

        let mut iter: usize = 0;
        let start = Instant::now();
        loop {
            let last = theta;
            let tmp = normalized.entries.iter().map(|DatasetEntry { km, price }| {
                // use the temporary t0 so we don't compute things twice
                let t0 = estimate_price(*km, theta) - price;
                (t0, t0 * km)
            }).reduce(|a, b| (a.0 + b.0, a.1 + b.1)).unwrap_or((0.0, 0.0));
            theta = (
                theta.0 - base * tmp.0,
                theta.1 - base * tmp.1,
            );
            iter += 1;
            if (last.0 - theta.0).abs() == 0.0 && (last.1 - theta.1).abs() == 0.0 {
                break;
            }
            // Cold function because we want the loop to be tight, thus not have all this garbage inlined. Reduces runtime by about 10%
            #[cold]
            fn print_info(iter: usize, start: &Instant, theta: (f64, f64)) {
                println!("{} iterations in {:.3}s", iter, start.elapsed().as_secs_f64());
                println!("Theta is currently {:?}", theta);
            }
            if iter & 0b1111111111111111111111111 == 0 {
                print_info(iter, &start, theta);
            }
        }

        println!("Done {} iterations in {:.3}s", iter, start.elapsed().as_secs_f64());
        //normalized.draw_to_file_with_theta("normalized.png", theta);
        //raw.draw_to_file_with_theta("raw.png", raw.denormalize_theta(theta));

        let _ = save_theta(theta_path, raw.denormalize_theta(theta));
    }
}