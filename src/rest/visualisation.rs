use chrono::{DateTime, Duration, Local, TimeZone};
use plotters::prelude::*;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

use crate::Configuration;

async fn get_data(
    filename: &str,
    from_date: DateTime<Local>,
    to_date: DateTime<Local>,
) -> Vec<(DateTime<Local>, i32)> {
    let mut buf = vec![];
    let data = File::open(filename).await.unwrap();
    let mut rdr = BufReader::new(data).lines();
    while let Ok(Some(line)) = rdr.next_line().await {
        let l = line
            .split(";")
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        match &l[..] {
            [date, value, ..] => {
                let date = Local.datetime_from_str(date, "%Y-%m-%d %H:%M:%S").unwrap();
                let watts = i32::from_str_radix(value.as_str(), 10).unwrap();
                if (date < to_date) && (date > from_date) {
                    buf.push((date, watts))
                }
            }
            _ => (),
        }
    }
    buf
}

pub(crate) async fn render_image(config: &Configuration) -> String {
    let (from_date, to_date) = (Local::now() - Duration::days(1), Local::now());

    let data = get_data(&config.log_location.to_str().unwrap(), from_date, to_date).await;
    let mut out_string = String::new();
    {
        let root = SVGBackend::with_string(&mut out_string, (1024, 768)).into_drawing_area(); // (OUT_FILE_NA).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .x_label_area_size(50)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .right_y_label_area_size(40)
            .build_cartesian_2d(from_date..to_date, -300..5000)
            .unwrap();

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperMiddle);
        chart
            .configure_mesh()
            .x_labels(6)
            .y_labels(20)
            .light_line_style(&WHITE)
            .x_label_formatter(&|v| format!("{:?}", v))
            .y_label_formatter(&|v| format!("{:?}", v))
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(data, GREEN.filled()))
            .unwrap();

        root.present().unwrap();
    }
    out_string
}
