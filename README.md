# aetolia

[![Build](https://github.com/EphyraSoftware/aetolia/actions/workflows/build.yaml/badge.svg)](https://github.com/EphyraSoftware/aetolia/actions/workflows/build.yaml)
[![coverage](https://shields.io/endpoint?url=https://ephyrasoftware.github.io/aetolia/coverage.json)](https://ephyrasoftware.github.io/aetolia/index.html)
![Crates.io Version](https://img.shields.io/crates/v/aetolia)

Calendar tools based on the open iCalendar standard format.

This library is still under development and not quite ready for use. It contains a parser, a builder, a validator and a serializer for iCalendar data.
There are still some unhandled cases and a lot of missing test coverage, which may reveal gaps in the implementation.

This library does not and will not provide the functionality of an iCalendar application, it is intended to be used to
build such applications.

## Examples

### Load a calendar from a file

```rust
use aetolia::prelude::*;

let calendar_file = std::fs::File::open("sample.ics").unwrap();
let parsed = load_ical(calendar_file).unwrap();

println!("Loaded calendar with {} objects", parsed.len());
```

### Validate a calendar

```rust
use aetolia::prelude::*;

let test_content = "BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:sample\r\n\
BEGIN:VEVENT\r\n\
DTSTAMP:20220101T000000Z\r\n\
UID:123\r\n\
UID:145\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n";

let parsed = load_ical(test_content.as_bytes()).unwrap();

let validation_errors = validate_model(&parsed[0]).unwrap();

for error in validation_errors {
    eprintln!("{}", error);
}
```

### Generate a calendar and serialize it

```rust
use aetolia::prelude::*;

let calendar = ICalObject::builder()
    .add_max_version("2.0")
    .finish_property()
    .add_product_id("sample")
    .finish_property()
    .add_event_component()
    .add_date_time_stamp(
        time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
        time::Time::from_hms(15, 0, 0).unwrap(),
    )
    .finish_property()
    .add_unique_identifier("test-id")
    .finish_property()
    .finish_component()
    .build();

let mut target = Vec::new();
calendar.write_model(&mut target).unwrap();
println!("{}", String::from_utf8(target).unwrap());
```
