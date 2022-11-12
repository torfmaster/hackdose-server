use chrono::{DateTime, Duration, Utc};
use tokio::sync::mpsc::Receiver;
use tplinker::devices::HS100;

use crate::Configuration;

struct ActorState {
    address: String,
    disable_threshold: isize,
    enable_threshold: isize,
    duration_minutes: usize,
    last_set: Option<DateTime<Utc>>,
}

pub(crate) async fn control_actors(rx: &mut Receiver<i32>, config: &Configuration) {
    let mut devs = config
        .actors
        .iter()
        .map(|actor| ActorState {
            address: actor.address.clone(),
            disable_threshold: actor.disable_threshold,
            enable_threshold: actor.enable_threshold,
            duration_minutes: actor.duration_minutes,
            last_set: None,
        })
        .collect::<Vec<_>>();

    use tplinker::capabilities::Switch;
    let mut on = false;

    // switch on random device
    let random_number = rand::random::<usize>() % devs.len();

    while let Some(received) = rx.recv().await {
        let dev = devs.get_mut(random_number).unwrap();
        let ActorState {
            address,
            disable_threshold,
            enable_threshold,
            duration_minutes,
            last_set,
        } = dev;

        let dev = HS100::new(&address);

        if let Ok(dev) = dev {
            let should_be_on = if !on {
                received < *enable_threshold as i32
            } else {
                !(received > *disable_threshold as i32)
            };
            if should_be_on != on {
                let now = chrono::Utc::now();
                if let Some(last_set_inner) = last_set {
                    let diff = now - *last_set_inner;
                    if diff > Duration::minutes(*duration_minutes as i64) {
                        on = should_be_on;
                        *last_set = Some(now.clone());
                        // naive debounce
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    }
                } else {
                    on = should_be_on;
                    *last_set = Some(chrono::Utc::now());
                }
            }
            if on {
                let _ = dev.switch_on();
            } else {
                let _ = dev.switch_off();
            }
        }
    }
}
