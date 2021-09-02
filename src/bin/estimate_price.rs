use ft_linear_regression::estimate_price::estimate_price;
use std::{env, io};
use ft_linear_regression::args::{arg_err, ArgParser};
use std::str::FromStr;
use std::cmp::max;
use ft_linear_regression::theta::{get_theta, ThetaFileArg};
use std::io::{Write, BufRead};


fn main() {
    let args: Vec<_> = env::args().skip(1).map(|it|it.to_lowercase()).collect();
    let mut used = vec![false; args.len()];
    let theta_path = ThetaFileArg::try_parse(&args, &mut used);
    let theta = get_theta(theta_path);

    let results: Vec<_> = args.iter().enumerate().filter_map(|(idx, arg)|{
        if !used[idx] {
            f64::from_str(arg).ok().map(|it| {
                used[idx] = true;
                it
            })
        } else {
            None
        }
    }).map(|it| (format!("{:.3} km", it), format!("{:.2} $", estimate_price(it, theta)))).collect();

    for (idx, it) in args.iter().enumerate() {
        if !used[idx] {
            arg_err(idx, it, "Arg is not recognized, ignoring. --help for more info");
        }
    }

    if results.is_empty() {
        println!("Please type in a float kilometer value, or exit to exit");
        print!("> ");
        let _ = io::stdout().flush();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(value) => {
                    if value == "exit" {
                        break
                    } else {
                        match f64::from_str(&value) {
                            Ok(value) => {
                                println!("{:.3} km is priced {:.2} $", value, estimate_price(value, theta))
                            }
                            Err(_) => {
                                println!("Value must be <float> or exit")
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("Error reading line: {}", err)
                }
            }
            print!("> ");
            let _ = io::stdout().flush();
        }
    } else {
        let max = results.iter().fold((0usize, 0usize), |(a, b), (sa, sb)| {
            (max(a, sa.len()), max(b, sb.len()))
        });

        for (kilometer, price) in results {
            println!("{0:>1$} is priced {2:>3$}", kilometer, max.0, price, max.1)
        }
    }

}