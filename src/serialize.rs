mod param;
mod property;
mod value;
mod component;
mod object;

use std::io::Write;

pub trait WriteModel {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::convert::ToModel;
    use crate::parser::Error;
    use crate::serialize::WriteModel;
    use crate::test_utils::check_rem;

    #[test]
    fn rtt_single_event() {
        let content = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTAMP:20211010T000000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        round_trip_ical_object(content);
    }

    fn round_trip_ical_object(content: &str) {
        let (rem, object) = crate::parser::ical_object::<Error>(content.as_bytes()).unwrap();
        check_rem(rem, 0);
        let model = object.to_model().unwrap();

        let mut buffer = Vec::new();
        model.write_model(&mut buffer).unwrap();
        let out_content = String::from_utf8_lossy(&buffer);

        similar_asserts::assert_eq!(content, out_content);
    }
}
