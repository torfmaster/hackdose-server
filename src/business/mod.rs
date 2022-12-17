use std::{collections::HashMap, sync::Arc};

use hackdose_sml_parser::{
    domain::{AnyValue, SmlMessages},
    obis::Obis,
};
use tokio::{
    io::AsyncWriteExt,
    sync::{mpsc::Sender, Mutex},
};
use tokio_stream::Stream;
use tokio_stream::StreamExt;

use crate::{smart_meter::body::find_watts, Configuration};

pub(crate) async fn handle_power_events(
    tx: &mut Sender<i32>,
    mutex: Arc<Mutex<HashMap<Obis, AnyValue>>>,
    config: &Configuration,
    mut power_events: impl Stream<Item = SmlMessages> + Unpin + Send + 'static,
) {
    while let Some(message) = power_events.next().await {
        let watts = find_watts(&message, mutex.clone()).await;

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
    }
}
