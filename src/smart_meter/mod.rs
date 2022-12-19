use tokio::io::AsyncRead;
use tokio_serial::SerialStream;

use crate::Configuration;

pub(crate) mod body;

pub(crate) fn uart_ir_sensor_data_stream(config: &Configuration) -> impl AsyncRead {
    let serial = tokio_serial::new(&config.ttys_location, 9600);
    let stream = SerialStream::open(&serial).unwrap();
    stream
}
