use std::{collections::HashMap, sync::Arc};

use hackdose_sml_parser::{
    domain::AnyValue,
    domain::{SmlMessageEnvelope, SmlMessages},
    obis::Obis,
};
use tokio::sync::Mutex;

trait Scale {
    fn scale(&self, scaler: i8) -> Self;
}

impl Scale for AnyValue {
    fn scale(&self, scaler: i8) -> Self {
        match self {
            AnyValue::Unsigned(v) => {
                let scaler = scaler;
                AnyValue::Unsigned((10f64.powf(scaler as f64) * *v as f64) as usize)
            }
            AnyValue::Signed(v) => {
                let scaler = scaler;
                AnyValue::Signed((10f64.powf(scaler as f64) * *v as f64) as isize)
            }
            AnyValue::String(v) => AnyValue::String(v.clone()),
        }
    }
}

pub async fn find_watts(
    messages: &SmlMessages,
    mutex: Arc<Mutex<HashMap<Obis, AnyValue>>>,
) -> Option<i32> {
    for list in &messages.messages {
        match list {
            SmlMessageEnvelope::GetOpenResponse(_) => continue,
            SmlMessageEnvelope::GetListResponse(body) => {
                let values = &body.value_list;
                let identified = values
                    .iter()
                    .flat_map(|value| {
                        Obis::from_number(&value.object_name)
                            .map(|x| (x, value.value.clone(), value.scaler.clone()))
                    })
                    .collect::<Vec<_>>();

                let mut value_list = mutex.lock().await;
                for (o, v, scaler) in identified.iter() {
                    value_list.insert(o.clone(), v.scale(scaler.unwrap_or(0)).clone());
                }

                let usage = identified
                    .iter()
                    .find(|(o, _, _)| o == &Obis::SumActiveInstantaneousPower)
                    .map(|(_, v, scaler)| v.scale(scaler.unwrap_or(0)));

                if let Some(usage) = usage {
                    if let AnyValue::Signed(value) = usage {
                        return Some(value as i32);
                    }
                }
            }
            SmlMessageEnvelope::GetCloseResponse => continue,
        }
    }
    return None;
}
