use std::{collections::HashMap, sync::Arc};

use futures::{future::BoxFuture, FutureExt};
use hackdose_sml_parser::{domain::AnyValue, obis::Obis};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc::Sender, Mutex},
};
use tokio_serial::SerialStream;

use crate::Configuration;

use self::{body::find_watts, envelope::SMLMessageBuilder};

pub(crate) mod body;
pub(crate) mod envelope;

pub(crate) async fn read_smart_meter(
    tx: &mut Sender<i32>,
    mutex: Arc<Mutex<HashMap<Obis, AnyValue>>>,
    config: &Configuration,
) {
    // serial
    let serial = tokio_serial::new(&config.ttys_location, 9600);
    let mut stream = SerialStream::open(&serial).unwrap();
    let mut buf = [0; 512];
    let mut builder = SMLMessageBuilder::Empty;

    while let Ok(n) = stream.read(&mut buf).await {
        handle_data(&mut builder, &buf[0..n], tx, mutex.clone(), config).await;
    }
}

pub(crate) fn handle_data<'a>(
    builder: &'a mut SMLMessageBuilder,
    buf: &'a [u8],
    tx: &'a mut Sender<i32>,
    mutex: Arc<Mutex<HashMap<Obis, AnyValue>>>,
    config: &'a Configuration,
) -> BoxFuture<'a, ()> {
    async move {
        builder.record(buf);

        match builder {
            SMLMessageBuilder::Complete { ref data, ref rest } => {
                let watts = find_watts(data, mutex.clone()).await;

                match watts {
                    Some(watts) => {
                        let time = chrono::Utc::now();
                        let f = time.format("%Y-%m-%d %H:%M:%S");
                        let log_line = format!("{};{}\n", f, watts);
                        let log = tokio::fs::OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(&config.log_location)
                            .await;
                        match log {
                            Ok(mut file) => {
                                let _ = file.write_all(log_line.as_bytes()).await;
                            }
                            Err(_) => (),
                        }
                        tx.send(watts).await.unwrap();
                    }
                    None => {}
                }
                if rest.len() == 0 {
                    *builder = SMLMessageBuilder::Empty;
                } else {
                    let rest = rest.clone();
                    *builder = SMLMessageBuilder::Empty;
                    handle_data(builder, &mut rest.clone(), tx, mutex.clone(), config).await;
                }
            }
            SMLMessageBuilder::Empty => (),
            SMLMessageBuilder::IncompleteStartSignature(_) => (),
            SMLMessageBuilder::Recording(_) => (),
        }
    }
    .boxed()
}
