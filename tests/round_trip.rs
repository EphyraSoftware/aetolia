use aetolia::prelude::*;

/// Create an ical object with as much coverage as possible, using the builder interface
/// Then that is built, serialized, and parsed to be compared with the original object.
///
/// Just to check that the content should be expected to be processed correctly, the object is
/// validated before being serialized and parsed.
#[test]
fn round_trip() {
    let object = ICalObject::builder()
        .add_calendar_scale("gregorian")
        .add_iana_param("scale-test", "test")
        .add_x_param("x-scale-test", "test")
        .finish_property()
        .add_method("publish")
        .add_iana_param("method-test", "test")
        .add_x_param("x-method-test", "test")
        .finish_property()
        .add_product_id("aetolia/test")
        .add_iana_param("product-id-test", "test")
        .add_x_param("x-product-id-test", "test")
        .finish_property()
        .add_max_version("2.0")
        .add_iana_param("max-version-test", "test")
        .add_x_param("x-max-version-test", "test")
        .finish_property()
        .add_event_component()
        .add_date_time_stamp(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .add_is_utc()
        .add_iana_param("date-time-stamp-test", "test")
        .add_x_param("x-date-time-stamp-test", "test")
        .finish_property()
        .add_unique_identifier("test-id")
        .add_iana_param("unique-id-test", "test")
        .add_x_param("x-unique-id-test", "test")
        .finish_property()
        .finish_component()
        .build();

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

    let parsed = load_ical(&target[..]).unwrap();

    assert_eq!(1, parsed.len());
    assert_eq!(object, parsed[0]);
}
