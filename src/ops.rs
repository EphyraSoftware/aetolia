use crate::convert::ToModel;
use crate::error::{AetoliaError, AetoliaResult};
use crate::model::object::ICalObject;
use crate::parser::{content_line_first_pass, ical_stream, Error};
use std::io::Read;

/// Load iCalendar data from a byte source.
///
/// If the input cannot be parsed or represented using the core model, an error is returned.
/// The content is expected to be a list of iCalendar objects. In most cases, this it is a single
/// iCalendar object, containing multiple components.
///
/// Note that this function does NOT validate the iCalendar data. You should use the validator if
/// you want to ensure that the data is reasonably correct. If you plan to ingest the data into
/// another system, you should validate the result of this function because the parser
/// and model permit a lot of inputs that could confuse other systems.
pub fn load_ical(input: impl AsRef<[u8]>) -> AetoliaResult<Vec<ICalObject>> {
    let (rem, content) = content_line_first_pass::<Error>(input.as_ref())
        .map_err(|e| AetoliaError::other(format!("First pass failed: {e}")))?;
    if !rem.is_empty() {
        return Err(AetoliaError::other("Trailing data after first pass"));
    }

    let (rem, stream) = ical_stream::<Error>(&content)
        .map_err(|e| AetoliaError::other(format!("Stream parsing failed: {e}")))?;
    if !rem.is_empty() {
        return Err(AetoliaError::other("Trailing data after stream"));
    }

    let model = stream.to_model()?;

    Ok(model)
}

/// Convenience function to load iCalendar data from a readable source.
///
/// The data is read to the end and then passed to [load_ical].
pub fn read_ical<R: Read>(mut input: R) -> AetoliaResult<Vec<ICalObject>> {
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    load_ical(buffer)
}
