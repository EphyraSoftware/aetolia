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

    #[test]
    fn rtt_journal() {
        let example_1 = "BEGIN:VCALENDAR\r\n\
BEGIN:VJOURNAL\r\n\
UID:19970901T130000Z-123405@example.com\r\n\
DTSTAMP:19970901T130000Z\r\n\
DTSTART;VALUE=DATE:19970317\r\n\
SUMMARY:Staff meeting minutes\r\n\
DESCRIPTION:1. Staff meeting: Participants include Joe\\, Lisa\\, and Bob. Aurora project plans were reviewed. There is currently no budget reserves for this project. Lisa will escalate to management. Next meeting on Tuesday.\\n 2. Telephone Conference: ABC Corp. sales representative called to discuss new printer. Promised to get us a demo by Friday.\\n3. Henry Miller (Handsoff Insurance): Car was totaled by tree. Is looking into a loaner car. 555-2323 (tel).\r\n\
END:VJOURNAL\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_1);
    }

    #[test]
    fn rtt_free_busy() {
        let example_1 = "BEGIN:VCALENDAR\r\n\
BEGIN:VFREEBUSY\r\n\
UID:19970901T082949Z-FA43EF@example.com\r\n\
ORGANIZER:mailto:jane_doe@example.com\r\n\
ATTENDEE:mailto:john_public@example.com\r\n\
DTSTART:19971015T050000Z\r\n\
DTEND:19971016T050000Z\r\n\
DTSTAMP:19970901T083000Z\r\n\
END:VFREEBUSY\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_1);

        let example_2 = "BEGIN:VCALENDAR\r\n\
BEGIN:VFREEBUSY\r\n\
UID:19970901T095957Z-76A912@example.com\r\n\
ORGANIZER:mailto:jane_doe@example.com\r\n\
ATTENDEE:mailto:john_public@example.com\r\n\
DTSTAMP:19970901T100000Z\r\n\
FREEBUSY:19971015T050000Z/PT8H30M,19971015T160000Z/PT5H30M,19971015T223000Z/PT6H30M\r\n\
URL:http://example.com/pub/busy/jpublic-01.ifb\r\n\
COMMENT:This iCalendar file contains busy time information for the next three months.\r\n\
END:VFREEBUSY\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_2);

        let example_3 = "BEGIN:VCALENDAR\r\n\
BEGIN:VFREEBUSY\r\n\
UID:19970901T115957Z-76A912@example.com\r\n\
DTSTAMP:19970901T120000Z\r\n\
ORGANIZER:jsmith@example.com\r\n\
DTSTART:19980313T141711Z\r\n\
DTEND:19980410T141711Z\r\n\
FREEBUSY:19980314T233000Z/19980315T003000Z\r\n\
FREEBUSY:19980316T153000Z/19980316T163000Z\r\n\
FREEBUSY:19980318T030000Z/19980318T040000Z\r\n\
URL:http://www.example.com/calendar/busytime/jsmith.ifb\r\n\
END:VFREEBUSY\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_3);
    }

    #[test]
    fn rtt_time_zone() {
        let example_1 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:America/New_York\r\n\
LAST-MODIFIED:20050809T050000Z\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19670430T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=4;BYDAY=-1SU;UNTIL=19730429T070000Z\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19671029T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU;UNTIL=20061029T060000Z\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19740106T020000\r\n\
RDATE:19750223T020000\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19760425T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=4;BYDAY=-1SU;UNTIL=19860427T070000Z\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19870405T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=4;BYDAY=1SU;UNTIL=20060402T070000Z\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:20070311T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:STANDARD\r\n\
DTSTART:20071104T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_1);

        let example_2 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:America/New_York\r\n\
LAST-MODIFIED:20050809T050000Z\r\n\
BEGIN:STANDARD\r\n\
DTSTART:20071104T020000\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:20070311T020000\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_2);

        let example_3 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:America/New_York\r\n\
LAST-MODIFIED:20050809T050000Z\r\n\
TZURL:http://zones.example.com/tz/America-New_York.ics\r\n\
BEGIN:STANDARD\r\n\
DTSTART:20071104T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:20070311T020000\r\n\
RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_3);

        let example_4 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:Fictitious\r\n\
LAST-MODIFIED:19870101T000000Z\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19671029T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19870405T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=4;UNTIL=19980404T070000Z\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_4);

        let example_5 = "BEGIN:VCALENDAR\r\n\
BEGIN:VTIMEZONE\r\n\
TZID:Fictitious\r\n\
LAST-MODIFIED:19870101T000000Z\r\n\
BEGIN:STANDARD\r\n\
DTSTART:19671029T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\n\
TZOFFSETFROM:-0400\r\n\
TZOFFSETTO:-0500\r\n\
TZNAME:EST\r\n\
END:STANDARD\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19870405T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=1SU;BYMONTH=4;UNTIL=19980404T070000Z\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
BEGIN:DAYLIGHT\r\n\
DTSTART:19990424T020000\r\n\
RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=4\r\n\
TZOFFSETFROM:-0500\r\n\
TZOFFSETTO:-0400\r\n\
TZNAME:EDT\r\n\
END:DAYLIGHT\r\n\
END:VTIMEZONE\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_5);
    }

    #[test]
    fn rtt_alarm() {
        let example_1 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
BEGIN:VALARM\r\n\
TRIGGER;VALUE=DATE-TIME:19970317T133000Z\r\n\
REPEAT:4\r\n\
DURATION:PT15M\r\n\
ACTION:AUDIO\r\n\
ATTACH;FMTTYPE=audio/basic:ftp://example.com/pub/sounds/bell-01.aud\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_1);

        let example_2 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
BEGIN:VALARM\r\n\
TRIGGER:-PT30M\r\n\
REPEAT:2\r\n\
DURATION:PT15M\r\n\
ACTION:DISPLAY\r\n\
DESCRIPTION:Breakfast meeting with executive\\n team at 8:30 AM EST.\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_2);

        let example_3 = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
BEGIN:VALARM\r\n\
TRIGGER;RELATED=END:-P2D\r\n\
ACTION:EMAIL\r\n\
ATTENDEE:mailto:john_doe@example.com\r\n\
SUMMARY:*** REMINDER: SEND AGENDA FOR WEEKLY STAFF MEETING ***\r\n\
DESCRIPTION:A draft agenda needs to be sent out to the attendees to the weekly managers meeting (MGR-LIST). Attached is a pointer the document template for the agenda file.\r\n\
ATTACH;FMTTYPE=application/msword:http://example.com/templates/agenda.doc\r\n\
END:VALARM\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

        round_trip_ical_object(example_3);
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
