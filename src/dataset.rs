use crate::args::FileParser;
use std::path::{Path};
use std::fs::{OpenOptions};
use std::io::{Read};
use std::fmt::Display;
use std::str::{FromStr, Lines};
use plotters::prelude::{BitMapBackend, WHITE, ChartBuilder, IntoFont, LineSeries, RED, PathElement, BLACK, PointSeries, EmptyElement};
use plotters::drawing::IntoDrawingArea;
use plotters::style::Color;
use plotters::element::Circle;
use crate::estimate_price::estimate_price;

pub struct DatasetArg;

impl FileParser<'_> for DatasetArg {
    const NAMES: &'static [&'static str] = &["-d", "--dataset"];
    const DESCRIPTION: &'static str = "The Learning Dataset, csv formatted with column headers";
}

#[derive(Copy, Clone)]
pub struct DatasetEntry {
    pub km: f64,
    pub price: f64,
}

#[derive(Clone)]
pub struct Dataset {
    pub entries: Vec<DatasetEntry>
}


impl Dataset {
    fn parse_header(headers: &str, separator: Option<&str>, dataset_file: impl Display) -> Result<(usize, usize), ()> {
        let headers = headers.split(separator.unwrap_or(",")).enumerate().fold((None, None), |(km, price), (idx, header)| {
            let mut known = false;
            let km = if header.eq_ignore_ascii_case("km") {
                known = true;
                if let Some(km) = km {
                    println!("Error: Duplicate column #{} in dataset {}: \"km\" column is already defined at index {}", idx, dataset_file, km);
                    Some(km)
                } else {
                    Some(idx)
                }
            } else {
                km
            };
            let price = if header.eq_ignore_ascii_case("price") {
                known = true;
                if let Some(price) = price {
                    println!("Error: Duplicate column #{} in dataset {}: \"price\" column is already defined at index {}", idx, dataset_file, price);
                    Some(price)
                } else {
                    Some(idx)
                }
            } else {
                price
            };
            if !known {
                println!("Warning: Unknown column \"{}\" #{} in dataset {}", idx, header, dataset_file);
            }
            (km, price)
        });

        match headers {
            (Some(km), Some(price)) => Ok((km, price)),
            (Some(_), None) => Err(println!("Error: Column \"price\" is missing in dataset {}", dataset_file)),
            (None, Some(_)) => Err(println!("Error: Column \"km\" is missing in dataset {}", dataset_file)),
            (None, None) => Err(println!("Error: Columns \"km\" and \"price\" are missing in dataset {}", dataset_file)),
        }
    }

    fn parse_lines(lines: Lines, (km, price): (usize, usize), separator: Option<&str>, dataset_file: impl Display) -> Result<Dataset, ()> {
        let entries = lines.enumerate().filter_map(|(idx, line)| {
            let idx = idx + 1;
            if line.is_empty() {
                return None;
            }
            let mut columns = line.split(separator.unwrap_or(","));
            let (km, price) = if km < price {
                (
                    columns.nth(km).and_then(|str| {
                        f64::from_str(str).map_err(|err| println!("Error: Row {} in dataset {} had bad km value in column {}: {}", idx, dataset_file, km, err)).ok()
                    }).or_else(|| {
                        println!("Error: Row {} in dataset {} is missing km value in column {}", idx, dataset_file, km);
                        None
                    })?,
                    columns.nth(price - km - 1).and_then(|str| {
                        f64::from_str(str).map_err(|err| println!("Error: Row {} in dataset {} had bad price value in column {}: {}", idx, dataset_file, price, err)).ok()
                    }).or_else(|| {
                        println!("Error: Row {} in dataset {} is missing price value in column {}", idx, dataset_file, price);
                        None
                    })?
                )
            } else {
                (
                    columns.nth(price).and_then(|str| {
                        f64::from_str(str).map_err(|err| println!("Error: Row {} in dataset {} had bad price value in column {}: {}", idx, dataset_file, price, err)).ok()
                    }).or_else(|| {
                        println!("Error: Row {} in dataset {} is missing price value in column {}", idx, dataset_file, price);
                        None
                    })?,
                    columns.nth(km - price - 1).and_then(|str| {
                        f64::from_str(str).map_err(|err| println!("Error: Row {} in dataset {} had bad km value in column {}: {}", idx, dataset_file, km, err)).ok()
                    }).or_else(|| {
                        println!("Error: Row {} in dataset {} is missing km value in column {}", idx, dataset_file, km);
                        None
                    })?
                )
            };
            Some(DatasetEntry {
                km,
                price,
            })
        }).collect();
        Ok(Dataset { entries })
    }

    pub fn read_from(path: Option<&Path>, separator: Option<&str>) -> Result<Dataset, ()> {
        let path = path.unwrap_or_else(|| Path::new("./data.csv"));
        match OpenOptions::new().read(true).open(path) {
            Ok(mut file) => {
                let mut string = String::new();
                file.read_to_string(&mut string).map_err(|err| println!("Error: Could not read dataset file {}: {}", path.display(), err))?;
                drop(file);
                let mut lines = string.lines();
                let headers = lines.next().ok_or_else(|| println!("Error: Dataset file {} is empty", path.display()))?;
                let headers = Self::parse_header(headers, separator, path.display())?;
                Self::parse_lines(lines, headers, separator, path.display())
            }
            Err(err) => {
                Err(println!("Error: Could not open dataset file {} with read permission: {}", path.display(), err))
            }
        }
    }

    fn gen_box(&self) -> ((f64, f64), (f64, f64)) {
        self.entries.iter().fold(((f64::MAX, f64::MIN), (f64::MAX, f64::MIN)), |a, b| ((a.0.0.min(b.km), a.0.1.max(b.km)), (a.1.0.min(b.price), a.1.1.max(b.price))))
    }

    pub fn normalize(mut self) -> Self {
        let ((kmmin, kmmax), (prmin, prmax)) = self.gen_box();
        self.entries.iter_mut().for_each(|it| {
            it.km = (it.km - kmmin) / (kmmax - kmmin);
            it.price = (it.price - prmin) / (prmax - prmin);
        });
        self
    }

    pub fn denormalize_theta(&self, theta: (f64, f64)) -> (f64, f64) {
        let ((kmmin, kmmax), (prmin, prmax)) = self.gen_box();
        let theta1 = theta.1 / (kmmax - kmmin) * (prmax - prmin);
        (theta.0 * (prmax - prmin) + prmin - kmmin * theta1, theta1)
    }

    pub fn draw_to_file_with_theta(&self, file: &str, theta: (f64, f64)) -> Result<(), Box<dyn std::error::Error>> {
        let ((kmmin, kmmax), (prmin, prmax)) = self.gen_box();
        let root = BitMapBackend::new(file, (1000, 1000)).into_drawing_area();

        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("Linear Regression", ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(50)
            .y_label_area_size(50)
            .build_cartesian_2d(kmmin..kmmax, prmin..prmax)?;

        chart.configure_mesh().draw()?;

        chart
            .draw_series(PointSeries::of_element(
                self.entries.iter().map(|it| (it.km, it.price)),
                2,
                &RED,
                &|c, s, st| {
                    return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                        + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                }
            ))?
            .label("data")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart.draw_series(LineSeries::new(
            vec![(kmmin, estimate_price(kmmin, theta)), (kmmax, estimate_price(kmmax, theta))],
            &RED,
        ))?;

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
        Ok(())
    }
}