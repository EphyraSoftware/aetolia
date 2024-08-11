use aetolia::prelude::*;

mod common;
use common::make_test_object;

/// Create an ical object with as much coverage as possible, using the builder interface
/// Then that is built, serialized, and parsed to be compared with the original object.
///
/// Just to check that the content should be expected to be processed correctly, the object is
/// validated before being serialized and parsed.
#[test]
fn round_trip() {
    let object = make_test_object();

    let validation_errors = validate_model(&object).unwrap();

    if !validation_errors.is_empty() {
        validation_errors.iter().for_each(|e| {
            eprintln!("{}", e);
        })
    }

    assert!(
        validation_errors.is_empty(),
        "Didn't expect any validation errors, see errors above"
    );

    let mut target = Vec::new();
    object.write_model(&mut target).unwrap();

    std::fs::write("tests/round_trip.ics", &target).unwrap();

    let parsed = load_ical(&target[..]).unwrap();

    assert_eq!(1, parsed.len());
    similar_asserts::assert_eq!(object, parsed[0]);
}

macro_rules! assert_model {
    ($equals:literal, $v:expr) => {
        let mut target = Vec::new();
        $v.write_model(&mut target).unwrap();
        let actual = String::from_utf8(target).unwrap();
        assert_eq!($equals, actual);
    };
}

#[test]
fn accessors() {
    let object = make_test_object();

    for component in object.components {
        match component {
            CalendarComponent::Event(event) => {
                let dt_stamp = event.get_property::<DateTimeStampProperty>().unwrap();
                assert_eq!(1, dt_stamp.get_iana_params("date-time-stamp-test").len());
                assert_eq!(1, dt_stamp.get_x_params("x-date-time-stamp-test").len());
                assert_model!("20240808T150000Z", dt_stamp.value());

                let uid = event.get_property::<UniqueIdentifierProperty>().unwrap();
                assert_eq!(1, uid.get_iana_params("unique-identifier-test").len());
                assert_eq!(1, uid.get_x_params("x-unique-identifier-test").len());
                assert_model!("test-id", uid.value());

                let dt_start = event.get_property::<DateTimeStartProperty>().unwrap();
                let tz_id_param = dt_start.get_param::<TimeZoneIdParam>().unwrap();
                assert_eq!("test", tz_id_param.tz_id);
                assert!(tz_id_param.unique);
                assert_eq!(1, dt_start.get_iana_params("date-time-start-test").len());
                assert_eq!(1, dt_start.get_x_params("x-date-time-start-test").len());
                assert_model!("20240808T150000", dt_start.value());

                let class = event.get_property::<ClassificationProperty>().unwrap();
                assert_eq!(1, class.get_iana_params("classification-test").len());
                assert_eq!(1, class.get_x_params("x-classification-test").len());
                assert_model!("PUBLIC", class.value());

                let attendees = event.get_properties::<AttendeeProperty>();
                assert_eq!(1, attendees.len());

                let alarms = event.alarms();
                assert_eq!(3, alarms.len());

                let trigger = alarms[0].get_property::<TriggerProperty>().unwrap();
                let trigger_relationship = trigger.get_param::<TriggerRelationshipParam>().unwrap();
                assert_eq!(
                    TriggerRelationship::Start,
                    trigger_relationship.trigger_relationship
                );
                assert_eq!(1, trigger.get_iana_params("trigger-test").len());
                assert_eq!(1, trigger.get_x_params("x-trigger-test").len());
                match trigger.value() {
                    TriggerValue::Relative(d) => {
                        assert_model!("P1D", d);
                    }
                    _ => {
                        panic!("Expected duration");
                    }
                }

                let iana_prop = event.get_iana_properties("other");
                assert_eq!(1, iana_prop.len());

                let x_prop = event.get_x_properties("x-other");
                assert_eq!(1, x_prop.len());
            }
            CalendarComponent::TimeZone(timezone) => {
                let tz_id = timezone.get_property::<TimeZoneIdProperty>().unwrap();
                assert_eq!(1, tz_id.get_iana_params("time-zone-id-test").len());
                assert_eq!(1, tz_id.get_x_params("x-time-zone-id-test").len());
                assert_eq!("test", tz_id.value().id);

                let nested_components = timezone.nested_components();
                assert_eq!(2, nested_components.len());

                let tz_name = nested_components[0].get_properties::<TimeZoneNameProperty>();
                assert_eq!(1, tz_name.len());
            }
            _ => {
                // More test coverage could be added here
            }
        }
    }
}
