mod component;
mod object;
mod param;
mod property;
mod value;

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

    // Check with data taken from RFC 5545, section 3.6.1
    #[test]
    fn rtt_event() {
        let example_1 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
UID:19970901T130000Z-123401@example.com\r\n\
DTSTAMP:19970901T130000Z\r\n\
DTSTART:19970903T163000Z\r\n\
DTEND:19970903T190000Z\r\n\
SUMMARY:Annual Employee Review\r\n\
CLASS:PRIVATE\r\n\
CATEGORIES:BUSINESS,HUMAN RESOURCES\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_1);

        let example_2 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
UID:19970901T130000Z-123402@example.com\r\n\
DTSTAMP:19970901T130000Z\r\n\
DTSTART:19970401T163000Z\r\n\
DTEND:19970402T010000Z\r\n\
SUMMARY:Laurel is in sensitivity awareness class.\r\n\
CLASS:PUBLIC\r\n\
CATEGORIES:BUSINESS,HUMAN RESOURCES\r\n\
TRANSP:TRANSPARENT\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_2);

        let example_3 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
UID:19970901T130000Z-123403@example.com\r\n\
DTSTAMP:19970901T130000Z\r\n\
DTSTART;VALUE=DATE:19971102\r\n\
SUMMARY:Our Blissful Anniversary\r\n\
TRANSP:TRANSPARENT\r\n\
CLASS:CONFIDENTIAL\r\n\
CATEGORIES:ANNIVERSARY,PERSONAL,SPECIAL OCCASION\r\n\
RRULE:FREQ=YEARLY\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_3);

        let example_4 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
UID:20070423T123432Z-541111@example.com\r\n\
DTSTAMP:20070423T123432Z\r\n\
DTSTART;VALUE=DATE:20070628\r\n\
DTEND;VALUE=DATE:20070709\r\n\
SUMMARY:Festival International de Jazz de Montreal\r\n\
TRANSP:TRANSPARENT\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_4);
    }

    // Check with data taken from RFC 5545, section 3.6.2
    #[test]
    fn rtt_to_do() {
        let example_1 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTODO\r\n\
UID:20070313T123432Z-456553@example.com\r\n\
DTSTAMP:20070313T123432Z\r\n\
DUE;VALUE=DATE:20070501\r\n\
SUMMARY:Submit Quebec Income Tax Return for 2006\r\n\
CLASS:CONFIDENTIAL\r\n\
CATEGORIES:FAMILY,FINANCE\r\n\
STATUS:NEEDS-ACTION\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_1);

        let example_2 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTODO\r\n\
UID:20070514T103211Z-123404@example.com\r\n\
DTSTAMP:20070514T103211Z\r\n\
DTSTART:20070514T110000Z\r\n\
DUE:20070709T130000Z\r\n\
COMPLETED:20070707T100000Z\r\n\
SUMMARY:Submit Revised Internet-Draft\r\n\
PRIORITY:1\r\n\
STATUS:NEEDS-ACTION\r\n\
END:VTODO\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_2);
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
