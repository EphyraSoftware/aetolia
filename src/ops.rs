use crate::convert::ToModel;
use crate::model::object::ICalObject;
use crate::parser::{content_line_first_pass, ical_stream, Error};
use std::io::Read;

/// Load iCalendar data from anything readable.
///
/// This could be a byte array, a file, a network stream, etc. If the file cannot be parsed or
/// represented using the core model, an error is returned.
/// The content is expected to be a list of iCalendar objects. In most cases, this it is a single
/// iCalendar object, containing multiple components.
///
/// Note that this function does NOT validate the iCalendar data. You should use the validator if
/// you want to ensure that the data is reasonably correct. If you plan to ingest the data into
/// another system, you should definitely validate the result of this function because the parser
/// and model permit a lot of inputs that could confuse other systems.
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
