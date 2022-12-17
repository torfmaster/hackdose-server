use futures::Stream;
use hackdose_sml_parser::{domain::SmlMessages, parser::parse_body};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::mpsc::{self, Sender},
};
use tokio_serial::SerialStream;
use tokio_stream::wrappers::ReceiverStream;

use crate::Configuration;

use self::envelope::SMLMessageBuilder;

pub(crate) mod body;
pub(crate) mod envelope;

pub(crate) fn uart_ir_sensor_data_stream(config: &Configuration) -> impl AsyncRead {
    let serial = tokio_serial::new(&config.ttys_location, 9600);
    let stream = SerialStream::open(&serial).unwrap();
    stream
}

pub(crate) fn sml_message_stream(
    mut stream: impl AsyncRead + Unpin + Send + 'static,
) -> impl Stream<Item = SmlMessages> {
    let (tx, rx) = mpsc::channel::<SmlMessages>(256);

    let mut buf = [0; 512];
    let mut builder = SMLMessageBuilder::Empty;

    tokio::spawn(async move {
        while let Ok(_) = stream.read(&mut buf).await {
            emit_message(&mut builder, &buf, tx.clone()).await;
        }
    });

    ReceiverStream::new(rx)
}

async fn emit_message<'a>(
    builder: &'a mut SMLMessageBuilder,
    buf: &'a [u8],
    tx: Sender<SmlMessages>,
) {
    let mut to_process = buf.to_vec();
    while to_process.len() > 0 {
        builder.record(&to_process);
        to_process = vec![];

        match builder {
            SMLMessageBuilder::Complete { ref data, ref rest } => {
                let result = parse_body(data);
                if let Ok(messages) = result {
                    let _ = tx.send(messages).await;
                }
                if rest.len() == 0 {
                    *builder = SMLMessageBuilder::Empty;
                } else {
                    to_process = rest.to_vec();
                    *builder = SMLMessageBuilder::Empty;
                }
            }
            SMLMessageBuilder::Empty => (),
            SMLMessageBuilder::IncompleteStartSignature(_) => (),
            SMLMessageBuilder::Recording(_) => (),
        }
    }
}
