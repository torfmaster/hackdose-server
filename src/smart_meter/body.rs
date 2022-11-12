use hackdose_sml_parser::{
    domain::AnyValue, domain::SmlMessageEnvelope, obis::Obis, parser::parse_body,
};

pub fn find_watts(body: &[u8]) -> Option<i32> {
    let result = parse_body(body);
    let result = result.ok()?;
    for list in result.messages {
        match list {
            SmlMessageEnvelope::GetOpenResponse(_) => continue,
            SmlMessageEnvelope::GetListResponse(body) => {
                let values = &body.value_list;
                let usage = values.iter().find(|value| {
                    value.object_name == Obis::SumActiveInstantaneousPower.obis_number()
                });

                if let Some(usage) = usage {
                    if let AnyValue::Signed(value) = usage.value {
                        return Some(value as i32);
                    }
                }
            }
            SmlMessageEnvelope::GetCloseResponse => continue,
        }
    }
    return None;
}
