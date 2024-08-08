use crate::convert::ToModel;
use crate::model::ICalObject;
use crate::parser::{content_line_first_pass, ical_stream, Error};
use std::io::Read;

pub fn load_ical<R: Read>(mut input: R) -> anyhow::Result<Vec<ICalObject>> {
    let mut content = Vec::new();
    input.read_to_end(&mut content)?;

    let (rem, content) = content_line_first_pass::<Error>(&content)
        .map_err(|e| anyhow::anyhow!("First pass failed: {:?}", e))?;
    if !rem.is_empty() {
        return Err(anyhow::anyhow!("Trailing data after first pass"));
    }

    let (rem, stream) = ical_stream::<Error>(&content)
        .map_err(|e| anyhow::anyhow!("Stream parsing failed: {:?}", e))?;
    if !rem.is_empty() {
        return Err(anyhow::anyhow!("Trailing data after stream"));
    }

    let model = stream.to_model()?;

    Ok(model)
}
