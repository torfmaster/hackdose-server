use business::handle_power_events;
use clap::Parser;
use hackdose_sml_parser::application::domain::AnyValue;
use hackdose_sml_parser::application::obis::Obis;
use hackdose_sml_parser::message_stream::sml_message_stream;
use serde::Deserialize;
use smart_meter::uart_ir_sensor_data_stream;
use std::sync::Arc;
use std::{collections::HashMap, path::PathBuf};
use tokio::fs::File;
use tokio::io::BufReader;

use actors::control_actors;
use gpio_cdev::{Chip, LineRequestFlags};
use rest::serve_rest_endpoint;
use tokio::io::AsyncReadExt;

mod actors;
mod business;
mod rest;
mod smart_meter;

#[derive(Deserialize, Clone)]
struct ActorConfiguration {
    address: String,
    disable_threshold: isize,
    enable_threshold: isize,
    duration_minutes: usize,
}

#[derive(Deserialize, Clone)]
pub(crate) struct Configuration {
    actors: Vec<ActorConfiguration>,
    log_location: PathBuf,
    gpio_location: String,
    ttys_location: String,
    gpio_power_pin: u32,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

#[tokio::main(worker_threads = 2)]
async fn main() {
    let args = Args::parse();

    let config = File::open(args.config).await.unwrap();
    let mut config_file = String::new();
    BufReader::new(config)
        .read_to_string(&mut config_file)
        .await
        .unwrap();
    let config = serde_yaml::from_str::<Configuration>(&config_file).unwrap();

    let mut chip = Chip::new(&config.gpio_location).unwrap();
    let output = chip.get_line(config.gpio_power_pin).unwrap();
    let output_handle = output
        .request(LineRequestFlags::OUTPUT, 0, "mirror-gpio")
        .unwrap();

    output_handle.set_value(1).unwrap();

    let stream = uart_ir_sensor_data_stream(&config);
    let power_events = sml_message_stream(stream);

    let (mut tx, mut rx) = tokio::sync::mpsc::channel::<i32>(100);
    let mutex = Arc::new(tokio::sync::Mutex::new(HashMap::<Obis, AnyValue>::new()));

    let mutex1 = mutex.clone();
    let mutex2 = mutex.clone();
    let config2 = config.clone();
    let config3 = config.clone();
    tokio::task::spawn(async move {
        handle_power_events(&mut tx, mutex1.clone(), &config.clone(), power_events).await
    });
    tokio::task::spawn(async move { control_actors(&mut rx, &config2.clone()).await });
    serve_rest_endpoint(mutex2.clone(), &config3.clone()).await;
}
