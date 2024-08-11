use aetolia::prelude::*;

pub fn make_test_object() -> ICalObject {
    let fancy_language_tag = LanguageTag::new("en")
        .with_ext_lang("tst")
        .with_script("Test")
        .with_region("US")
        .add_variant("test1")
        .add_variant("test2")
        .add_extension("u-co-phonebk")
        .add_extension("u-nu-latn")
        .with_private_use("x-TST");

    ICalObject::builder()
        // RFC 5545: 3.7.1
        .add_calendar_scale("gregorian")
        .add_iana_param("scale-test", "test")
        .add_x_param("x-scale-test", "test")
        .finish_property()
        // RFC 5545: 3.7.2
        .add_method("publish")
        .add_iana_param("method-test", "test")
        .add_x_param("x-method-test", "test")
        .finish_property()
        // RFC 5545: 3.7.3
        .add_product_id("aetolia/test")
        .add_iana_param("product-id-test", "test")
        .add_x_param("x-product-id-test", "test")
        .finish_property()
        // RFC 5545: 3.7.4
        .add_max_version("2.0")
        .add_iana_param("max-version-test", "test")
        .add_x_param("x-max-version-test", "test")
        .finish_property()
        // RFC 5545: 3.6.1 - with properties added in the order they appear in the spec
        .add_event_component()
        // RFC 5545: 3.8.7.2
        .add_date_time_stamp(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("date-time-stamp-test", "test")
        .add_x_param("x-date-time-stamp-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.7
        .add_unique_identifier("test-id")
        .add_iana_param("unique-identifier-test", "test")
        .add_x_param("x-unique-identifier-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.4
        .add_date_time_start(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_iana_param("date-time-start-test", "test")
        .add_x_param("x-date-time-start-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.3
        .add_classification(Classification::Public)
        .add_iana_param("classification-test", "test")
        .add_x_param("x-classification-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.1
        .add_date_time_created(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .add_iana_param("created-test", "test")
        .add_x_param("x-created-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.5
        .add_description("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("description-test", "test")
        .add_x_param("x-description-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.6
        .add_geographic_position(3.111, 4.2)
        .add_iana_param("geo-test", "test")
        .add_x_param("x-geo-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.3
        .add_last_modified(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("last-modified-test", "test")
        .add_x_param("x-last-modified-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.7
        .add_location("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("location-test", "test")
        .add_x_param("x-location-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.3
        .add_organizer("mailto:hello@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("http://example.com/test")
        .add_sent_by("mailto:hello@test.net")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("organizer-test", "test")
        .add_x_param("x-organizer-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.9
        .add_priority(1)
        .add_iana_param("priority-test", "test")
        .add_x_param("x-priority-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.4
        .add_sequence(300)
        .add_iana_param("sequence-test", "test")
        .add_x_param("x-sequence-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.11
        .add_status(StatusEvent::Confirmed)
        .add_iana_param("status-test", "test")
        .add_x_param("x-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.12
        .add_summary("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("summary-test", "test")
        .add_x_param("x-summary-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.7
        .add_time_transparency(TimeTransparency::Opaque)
        .add_iana_param("time-transparency-test", "test")
        .add_x_param("x-time-transparency-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.6
        .add_url("http://example.com/test")
        .add_iana_param("url-test", "test")
        .add_x_param("x-url-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.4
        .add_recurrence_id(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_range(Range::ThisAndFuture)
        .add_iana_param("recurrence-id-test", "test")
        .add_x_param("x-recurrence-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.3
        .add_recurrence_rule(RecurFreq::Yearly, |rule| {
            // Likely a junk rule, it's just to check that everything is settable and round trips correctly
            rule.set_until(
                (
                    time::Date::from_calendar_date(2035, time::Month::August, 20).unwrap(),
                    time::Time::from_hms(15, 0, 0).unwrap(),
                    true,
                )
                    .into(),
            )
            .set_count(1_000)
            .set_interval(4)
            .set_by_second(vec![1, 2, 3])
            .set_by_minute(vec![1, 2, 3])
            .set_by_hour(vec![1, 2, 3])
            .set_by_day(vec![OffsetWeekday::new(Weekday::Monday, None)])
            .set_by_month_day(vec![1, 2, 3])
            .set_by_year_day(vec![1, 2, 3])
            .set_by_week_number(vec![1, 2, 3])
            .set_by_month(vec![
                time::Month::January,
                time::Month::February,
                time::Month::March,
            ])
            .set_by_set_pos(vec![1, 2, 3])
            .set_week_start(Weekday::Monday)
        })
        .add_iana_param("recurrence-rule-test", "test")
        .add_x_param("x-recurrence-rule-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.2
        .add_date_time_end(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_iana_param("date-time-end-test", "test")
        .add_x_param("x-date-time-end-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.1
        .add_attach_uri("http://example.com/test")
        .add_fmt_type("text", "xml")
        .add_iana_param("attach-test", "test")
        .add_x_param("x-attach-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.1
        .add_attendee("http:video-call.test.net")
        .add_calendar_user_type(CalendarUserType::Room)
        .add_members(vec!["video:video-call.test.net"])
        .add_role(Role::NonParticipant)
        .add_participation_status(ParticipationStatusEvent::Accepted)
        .add_rsvp()
        .add_delegated_to(vec!["http:video-call-1.test.net"])
        .add_delegated_from(vec!["http:video-call-2.test.net"])
        .add_sent_by("mailto:info@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("CID:hello")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("attendee-test", "test")
        .add_x_param("x-attendee-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.2
        .add_categories(vec!["test1", "test2", "test3"])
        .add_language(fancy_language_tag.clone())
        .add_iana_param("categories-test", "test")
        .add_x_param("x-categories-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.4
        .add_comment("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("comment-test", "test")
        .add_x_param("x-comment-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.2
        .add_contact("mailto:admin@test.net")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("contact-test", "test")
        .add_x_param("x-contact-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.1
        .add_exception_date_times(vec![
            (
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
            (
                time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
        ])
        .add_tz_id("test", true)
        .add_iana_param("exception-date-times-test", "test")
        .add_x_param("x-exception-date-times-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.3
        .add_request_status(&[2, 1], "Success", Some("great event"))
        .add_language(fancy_language_tag.clone())
        .add_iana_param("request-status-test", "test")
        .add_x_param("x-request-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.5
        .add_related_to("some-other-event")
        .add_relationship_type(RelationshipType::Parent)
        .add_iana_param("related-to-test", "test")
        .add_x_param("x-related-to-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.10
        .add_resources(vec!["test1", "test2", "test3"])
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("resources-test", "test")
        .add_x_param("x-resources-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.2
        .add_recurrence_date_date_times(vec![
            (
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
            (
                time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
        ])
        .add_tz_id("test", true)
        .add_iana_param("recurrence-date-test", "test")
        .add_x_param("x-recurrence-date-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.1
        .add_iana_property("other", "some-value")
        .add_iana_param("other-test", "test")
        .add_x_param("x-other-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.2
        .add_x_property("x-other", "some-value")
        .add_iana_param("x-other-test", "test")
        .add_x_param("x-x-other-test", "test")
        .finish_property()
        .add_audio_alarm()
        // RFC 5545: 3.8.6.1
        .add_action()
        .add_iana_param("action-test", "test")
        .add_x_param("x-action-test", "test")
        .finish_property()
        // RFC 5545: 3.8.6.3
        .add_relative_trigger(Duration::days(1, 1))
        .add_trigger_relationship(TriggerRelationship::Start)
        .add_iana_param("trigger-test", "test")
        .add_x_param("x-trigger-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.5
        .add_duration(|| Duration::hours(1, 1).build())
        .add_iana_param("duration-test", "test")
        .add_x_param("x-duration-test", "test")
        .finish_property()
        // RFC 5545: 3.8.6.2
        .add_repeat(2)
        .add_iana_param("repeat-test", "test")
        .add_x_param("x-repeat-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.1
        .add_attach_binary("YXNkZg==")
        .add_iana_param("attach-test", "test")
        .add_x_param("x-attach-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.1
        .add_iana_property("other", "some-value")
        .add_iana_param("other-test", "test")
        .add_x_param("x-other-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.2
        .add_x_property("x-other", "some-value")
        .add_iana_param("x-other-test", "test")
        .add_x_param("x-x-other-test", "test")
        .finish_property()
        .finish_component()
        .add_display_alarm()
        .add_action()
        .finish_property()
        .add_description("test")
        .finish_property()
        // RFC 5545: 3.8.6.3
        .add_absolute_trigger(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .add_iana_param("trigger-test", "test")
        .add_x_param("x-trigger-test", "test")
        .finish_property()
        .add_duration(|| Duration::hours(1, 1).build())
        .finish_property()
        .add_repeat(2)
        .finish_property()
        .add_iana_property("other", "some-value")
        .finish_property()
        .add_x_property("x-other", "some-value")
        .finish_property()
        .finish_component()
        .add_email_alarm()
        .add_action()
        .finish_property()
        .add_description("test")
        .finish_property()
        .add_relative_trigger(Duration::days(1, 1))
        .finish_property()
        .add_summary("test")
        .finish_property()
        .add_attendee("mailto:notify@test.net")
        .finish_property()
        .add_duration(|| Duration::hours(1, 1).build())
        .finish_property()
        .add_repeat(2)
        .finish_property()
        .add_attach_uri("http://example.com/test")
        .finish_property()
        .add_iana_property("other", "some-value")
        .finish_property()
        .add_x_property("x-other-test", "test")
        .finish_property()
        .finish_component()
        .finish_component()
        .add_to_do_component()
        // RFC 5545: 3.8.7.2
        .add_date_time_stamp(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("date-time-stamp-test", "test")
        .add_x_param("x-date-time-stamp-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.7
        .add_unique_identifier("test-id")
        .add_iana_param("unique-id-test", "test")
        .add_x_param("x-unique-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.3
        .add_classification(Classification::Public)
        .add_iana_param("class-test", "test")
        .add_x_param("x-class-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.1
        .add_date_time_completed(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("completed-test", "test")
        .add_x_param("x-completed-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.1
        .add_date_time_created(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .add_iana_param("created-test", "test")
        .add_x_param("x-created-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.5
        .add_description("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("description-test", "test")
        .add_x_param("x-description-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.4
        .add_date_time_start(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_iana_param("date-time-start-test", "test")
        .add_x_param("x-date-time-start-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.6
        .add_geographic_position(3.111, 4.2)
        .add_iana_param("geo-test", "test")
        .add_x_param("x-geo-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.3
        .add_last_modified(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("last-modified-test", "test")
        .add_x_param("x-last-modified-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.7
        .add_location("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("location-test", "test")
        .add_x_param("x-location-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.3
        .add_organizer("mailto:hello@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("http://example.com/test")
        .add_sent_by("mailto:hello@test.net")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("organizer-test", "test")
        .add_x_param("x-organizer-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.8
        .add_percent_complete(50)
        .add_iana_param("percent-complete-test", "test")
        .add_x_param("x-percent-complete-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.9
        .add_priority(5)
        .add_iana_param("priority-test", "test")
        .add_x_param("x-priority-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.4
        .add_recurrence_id(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_range(Range::ThisAndFuture)
        .add_iana_param("recurrence-id-test", "test")
        .add_x_param("x-recurrence-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.4
        .add_sequence(300)
        .add_iana_param("sequence-test", "test")
        .add_x_param("x-sequence-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.11
        .add_status(StatusToDo::InProcess)
        .add_iana_param("status-test", "test")
        .add_x_param("x-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.12
        .add_summary("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("summary-test", "test")
        .add_x_param("x-summary-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.6
        .add_url("http://example.com/test")
        .add_iana_param("url-test", "test")
        .add_x_param("x-url-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.3
        .add_recurrence_rule(RecurFreq::Yearly, |rule| {
            // Likely a junk rule, it's just to check that everything is settable and round trips correctly
            rule.set_until(
                (
                    time::Date::from_calendar_date(2035, time::Month::August, 20).unwrap(),
                    time::Time::from_hms(15, 0, 0).unwrap(),
                    true,
                )
                    .into(),
            )
            .set_count(1_000)
            .set_interval(4)
            .set_by_second(vec![1, 2, 3])
            .set_by_minute(vec![1, 2, 3])
            .set_by_hour(vec![1, 2, 3])
            .set_by_day(vec![OffsetWeekday::new(Weekday::Monday, None)])
            .set_by_month_day(vec![1, 2, 3])
            .set_by_year_day(vec![1, 2, 3])
            .set_by_week_number(vec![1, 2, 3])
            .set_by_month(vec![
                time::Month::January,
                time::Month::February,
                time::Month::March,
            ])
            .set_by_set_pos(vec![1, 2, 3])
            .set_week_start(Weekday::Monday)
        })
        .add_iana_param("recurrence-rule-test", "test")
        .add_x_param("x-recurrence-rule-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.5
        .add_duration(|| Duration::hours(1, 1).build())
        .add_iana_param("duration-test", "test")
        .add_x_param("x-duration-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.1
        .add_attach_uri("http://example.com/test")
        .add_fmt_type("text", "xml")
        .add_iana_param("attach-test", "test")
        .add_x_param("x-attach-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.1
        .add_attendee("http:video-call.test.net")
        .add_calendar_user_type(CalendarUserType::Room)
        .add_members(vec!["video:video-call.test.net"])
        .add_role(Role::NonParticipant)
        .add_participation_status(ParticipationStatusToDo::NeedsAction)
        .add_rsvp()
        .add_delegated_to(vec!["http:video-call-1.test.net"])
        .add_delegated_from(vec!["http:video-call-2.test.net"])
        .add_sent_by("mailto:info@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("CID:hello")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("attendee-test", "test")
        .add_x_param("x-attendee-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.2
        .add_categories(vec!["test1", "test2", "test3"])
        .add_language(fancy_language_tag.clone())
        .add_iana_param("categories-test", "test")
        .add_x_param("x-categories-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.4
        .add_comment("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("comment-test", "test")
        .add_x_param("x-comment-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.2
        .add_contact("mailto:admin@test.net")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("contact-test", "test")
        .add_x_param("x-contact-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.1
        .add_exception_date_times(vec![
            (
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
            (
                time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
        ])
        .add_tz_id("test", true)
        .add_iana_param("exception-date-times-test", "test")
        .add_x_param("x-exception-date-times-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.3
        .add_request_status(&[2, 1], "Success", Some("great event"))
        .add_language(fancy_language_tag.clone())
        .add_iana_param("request-status-test", "test")
        .add_x_param("x-request-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.5
        .add_related_to("some-other-event")
        .add_relationship_type(RelationshipType::Parent)
        .add_iana_param("related-to-test", "test")
        .add_x_param("x-related-to-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.10
        .add_resources(vec!["test1", "test2", "test3"])
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("resources-test", "test")
        .add_x_param("x-resources-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.2
        .add_recurrence_date_date_times(vec![
            (
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
            (
                time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
        ])
        .add_tz_id("test", true)
        .add_iana_param("recurrence-date-test", "test")
        .add_x_param("x-recurrence-date-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.1
        .add_iana_property("other", "some-value")
        .add_iana_param("other-test", "test")
        .add_x_param("x-other-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.2
        .add_x_property("x-other", "some-value")
        .add_iana_param("x-other-test", "test")
        .add_x_param("x-x-other-test", "test")
        .finish_property()
        .add_display_alarm()
        .add_action()
        .finish_property()
        .add_description("test")
        .finish_property()
        // RFC 5545: 3.8.6.3
        .add_absolute_trigger(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .add_iana_param("trigger-test", "test")
        .add_x_param("x-trigger-test", "test")
        .finish_property()
        .add_duration(|| Duration::hours(1, 1).build())
        .finish_property()
        .add_repeat(2)
        .finish_property()
        .add_iana_property("other", "some-value")
        .finish_property()
        .add_x_property("x-other", "some-value")
        .finish_property()
        .finish_component()
        .finish_component()
        .add_journal_component()
        // RFC 5545: 3.8.7.2
        .add_date_time_stamp(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("date-time-stamp-test", "test")
        .add_x_param("x-date-time-stamp-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.7
        .add_unique_identifier("test-id")
        .add_iana_param("unique-id-test", "test")
        .add_x_param("x-unique-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.3
        .add_classification(Classification::Public)
        .add_iana_param("class-test", "test")
        .add_x_param("x-class-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.1
        .add_date_time_created(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .add_iana_param("created-test", "test")
        .add_x_param("x-created-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.4
        .add_date_time_start(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_iana_param("date-time-start-test", "test")
        .add_x_param("x-date-time-start-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.3
        .add_last_modified(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("last-modified-test", "test")
        .add_x_param("x-last-modified-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.3
        .add_organizer("mailto:hello@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("http://example.com/test")
        .add_sent_by("mailto:hello@test.net")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("organizer-test", "test")
        .add_x_param("x-organizer-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.4
        .add_recurrence_id(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .add_tz_id("test", true)
        .add_range(Range::ThisAndFuture)
        .add_iana_param("recurrence-id-test", "test")
        .add_x_param("x-recurrence-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.4
        .add_sequence(300)
        .add_iana_param("sequence-test", "test")
        .add_x_param("x-sequence-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.11
        .add_status(StatusJournal::Draft)
        .add_iana_param("status-test", "test")
        .add_x_param("x-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.12
        .add_summary("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("summary-test", "test")
        .add_x_param("x-summary-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.6
        .add_url("http://example.com/test")
        .add_iana_param("url-test", "test")
        .add_x_param("x-url-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.3
        .add_recurrence_rule(RecurFreq::Yearly, |rule| {
            // Likely a junk rule, it's just to check that everything is settable and round trips correctly
            rule.set_until(
                (
                    time::Date::from_calendar_date(2035, time::Month::August, 20).unwrap(),
                    time::Time::from_hms(15, 0, 0).unwrap(),
                    true,
                )
                    .into(),
            )
            .set_count(1_000)
            .set_interval(4)
            .set_by_second(vec![1, 2, 3])
            .set_by_minute(vec![1, 2, 3])
            .set_by_hour(vec![1, 2, 3])
            .set_by_day(vec![OffsetWeekday::new(Weekday::Monday, None)])
            .set_by_month_day(vec![1, 2, 3])
            .set_by_year_day(vec![1, 2, 3])
            .set_by_week_number(vec![1, 2, 3])
            .set_by_month(vec![
                time::Month::January,
                time::Month::February,
                time::Month::March,
            ])
            .set_by_set_pos(vec![1, 2, 3])
            .set_week_start(Weekday::Monday)
        })
        .add_iana_param("recurrence-rule-test", "test")
        .add_x_param("x-recurrence-rule-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.1
        .add_attach_uri("http://example.com/test")
        .add_fmt_type("text", "xml")
        .add_iana_param("attach-test", "test")
        .add_x_param("x-attach-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.1
        .add_attendee("http:video-call.test.net")
        .add_calendar_user_type(CalendarUserType::Room)
        .add_members(vec!["video:video-call.test.net"])
        .add_role(Role::NonParticipant)
        .add_participation_status(ParticipationStatusJournal::NeedsAction)
        .add_rsvp()
        .add_delegated_to(vec!["http:video-call-1.test.net"])
        .add_delegated_from(vec!["http:video-call-2.test.net"])
        .add_sent_by("mailto:info@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("CID:hello")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("attendee-test", "test")
        .add_x_param("x-attendee-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.2
        .add_categories(vec!["test1", "test2", "test3"])
        .add_language(fancy_language_tag.clone())
        .add_iana_param("categories-test", "test")
        .add_x_param("x-categories-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.4
        .add_comment("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("comment-test", "test")
        .add_x_param("x-comment-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.2
        .add_contact("mailto:admin@test.net")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("contact-test", "test")
        .add_x_param("x-contact-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.5
        .add_description("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("description-test", "test")
        .add_x_param("x-description-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.1
        .add_exception_date_times(vec![
            (
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
            (
                time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
        ])
        .add_tz_id("test", true)
        .add_iana_param("exception-date-times-test", "test")
        .add_x_param("x-exception-date-times-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.5
        .add_related_to("some-other-event")
        .add_relationship_type(RelationshipType::Parent)
        .add_iana_param("related-to-test", "test")
        .add_x_param("x-related-to-test", "test")
        .finish_property()
        // RFC 5545: 3.8.5.2
        .add_recurrence_date_date_times(vec![
            (
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
            (
                time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
                true,
            )
                .into(),
        ])
        .add_tz_id("test", true)
        .add_iana_param("recurrence-date-test", "test")
        .add_x_param("x-recurrence-date-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.3
        .add_request_status(&[2, 1], "Success", Some("great event"))
        .add_language(fancy_language_tag.clone())
        .add_iana_param("request-status-test", "test")
        .add_x_param("x-request-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.1
        .add_iana_property("other", "some-value")
        .add_iana_param("other-test", "test")
        .add_x_param("x-other-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.2
        .add_x_property("x-other", "some-value")
        .add_iana_param("x-other-test", "test")
        .add_x_param("x-x-other-test", "test")
        .finish_property()
        .finish_component()
        .add_free_busy_component()
        // RFC 5545: 3.8.7.2
        .add_date_time_stamp(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("date-time-stamp-test", "test")
        .add_x_param("x-date-time-stamp-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.7
        .add_unique_identifier("test-id")
        .add_iana_param("unique-id-test", "test")
        .add_x_param("x-unique-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.2
        .add_contact("mailto:admin@test.net")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("contact-test", "test")
        .add_x_param("x-contact-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.4
        .add_date_time_start(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .set_is_utc()
        .add_iana_param("date-time-start-test", "test")
        .add_x_param("x-date-time-start-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.2
        .add_date_time_end(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            Some(time::Time::from_hms(15, 0, 0).unwrap()),
        )
        .set_is_utc()
        .add_iana_param("date-time-end-test", "test")
        .add_x_param("x-date-time-end-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.3
        .add_organizer("mailto:hello@test.net")
        .add_common_name("test")
        .add_directory_entry_reference("http://example.com/test")
        .add_sent_by("mailto:hello@test.net")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("organizer-test", "test")
        .add_x_param("x-organizer-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.6
        .add_url("http://example.com/test")
        .add_iana_param("url-test", "test")
        .add_x_param("x-url-test", "test")
        .finish_property()
        // RFC 5545: 3.8.4.1
        .add_attendee("http:video-call.test.net")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("attendee-test", "test")
        .add_x_param("x-attendee-test", "test")
        .finish_property()
        // RFC 5545: 3.8.1.4
        .add_comment("test")
        .add_alternate_representation("http://example.com/test")
        .add_language(fancy_language_tag.clone())
        .add_iana_param("comment-test", "test")
        .add_x_param("x-comment-test", "test")
        .finish_property()
        // RFC 5545: 3.8.2.6
        .add_free_busy_time(
            FreeBusyTimeType::BusyTentative,
            vec![
                Period::new_explicit(
                    time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                    time::Time::from_hms(15, 0, 0).unwrap(),
                    time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                    time::Time::from_hms(15, 0, 0).unwrap(),
                    true,
                ),
                Period::new_start(
                    time::Date::from_calendar_date(2024, time::Month::August, 10).unwrap(),
                    time::Time::from_hms(15, 0, 0).unwrap(),
                    true,
                    Duration::weeks(1, 3),
                ),
            ],
        )
        .add_iana_param("free-busy-time-test", "test")
        .add_x_param("x-free-busy-time-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.3
        .add_request_status(&[2, 1], "Success", Some("great event"))
        .add_language(fancy_language_tag.clone())
        .add_iana_param("request-status-test", "test")
        .add_x_param("x-request-status-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.1
        .add_iana_property("other", "some-value")
        .add_iana_param("other-test", "test")
        .add_x_param("x-other-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.2
        .add_x_property("x-other", "some-value")
        .add_iana_param("x-other-test", "test")
        .add_x_param("x-x-other-test", "test")
        .finish_property()
        .finish_component()
        .add_time_zone_component()
        // RFC 5545: 3.8.3.1
        .add_time_zone_id("test", false)
        .add_iana_param("time-zone-id-test", "test")
        .add_x_param("x-time-zone-id-test", "test")
        .finish_property()
        // RFC 5545: 3.8.7.3
        .add_last_modified(
            time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
            time::Time::from_hms(15, 0, 0).unwrap(),
        )
        .set_is_utc()
        .add_iana_param("last-modified-test", "test")
        .add_x_param("x-last-modified-test", "test")
        .finish_property()
        // RFC 5545: 3.8.3.5
        .add_time_zone_url("http://example.com/test")
        .add_iana_param("time-zone-url-test", "test")
        .add_x_param("x-time-zone-url-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.1
        .add_iana_property("other", "some-value")
        .add_iana_param("other-test", "test")
        .add_x_param("x-other-test", "test")
        .finish_property()
        // RFC 5545: 3.8.8.2
        .add_x_property("x-other", "some-value")
        .add_iana_param("x-other-test", "test")
        .add_x_param("x-x-other-test", "test")
        .finish_property()
        .add_standard_time(|b| {
            // RFC 5545: 3.8.2.4
            b.add_date_time_start(
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
            )
            .add_iana_param("date-time-start-test", "test")
            .add_x_param("x-date-time-start-test", "test")
            .finish_property()
            // RFC 5545: 3.8.3.4
            .add_time_zone_offset_to(TimeZoneOffset::new(1, 5, 0, None))
            .add_iana_param("time-zone-offset-to-test", "test")
            .add_x_param("x-time-zone-offset-to-test", "test")
            .finish_property()
            // RFC 5545: 3.8.3.3
            .add_time_zone_offset_from(TimeZoneOffset::new(-1, 5, 0, None))
            .add_iana_param("time-zone-offset-from-test", "test")
            .add_x_param("x-time-zone-offset-from-test", "test")
            .finish_property()
            // RFC 5545: 3.8.5.3
            .add_recurrence_rule(RecurFreq::Yearly, |rule| {
                // Likely a junk rule, it's just to check that everything is settable and round trips correctly
                rule.set_until(
                    (
                        time::Date::from_calendar_date(2035, time::Month::August, 20).unwrap(),
                        time::Time::from_hms(15, 0, 0).unwrap(),
                        true,
                    )
                        .into(),
                )
                .set_count(1_000)
                .set_interval(4)
                .set_by_second(vec![1, 2, 3])
                .set_by_minute(vec![1, 2, 3])
                .set_by_hour(vec![1, 2, 3])
                .set_by_day(vec![OffsetWeekday::new(Weekday::Monday, None)])
                .set_by_month_day(vec![1, 2, 3])
                .set_by_year_day(vec![1, 2, 3])
                .set_by_week_number(vec![1, 2, 3])
                .set_by_month(vec![
                    time::Month::January,
                    time::Month::February,
                    time::Month::March,
                ])
                .set_by_set_pos(vec![1, 2, 3])
                .set_week_start(Weekday::Monday)
            })
            .add_iana_param("recurrence-rule-test", "test")
            .add_x_param("x-recurrence-rule-test", "test")
            .finish_property()
            // RFC 5545: 3.8.1.4
            .add_comment("test")
            .add_alternate_representation("http://example.com/test")
            .add_language(LanguageTag::new("en").with_region("US"))
            .add_iana_param("comment-test", "test")
            .add_x_param("x-comment-test", "test")
            .finish_property()
            // RFC 5545: 3.8.5.2
            .add_recurrence_date_date_times(vec![
                (
                    time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                    Some(time::Time::from_hms(15, 0, 0).unwrap()),
                    true,
                )
                    .into(),
                (
                    time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                    Some(time::Time::from_hms(15, 0, 0).unwrap()),
                    true,
                )
                    .into(),
            ])
            .add_tz_id("test", true)
            .add_iana_param("recurrence-date-test", "test")
            .add_x_param("x-recurrence-date-test", "test")
            .finish_property()
            .add_time_zone_name("test")
            .add_iana_param("time-zone-name-test", "test")
            .add_x_param("x-time-zone-name-test", "test")
            .finish_property()
            // RFC 5545: 3.8.8.1
            .add_iana_property("other", "some-value")
            .add_iana_param("other-test", "test")
            .add_x_param("x-other-test", "test")
            .finish_property()
            // RFC 5545: 3.8.8.2
            .add_x_property("x-other", "some-value")
            .add_iana_param("x-other-test", "test")
            .add_x_param("x-x-other-test", "test")
            .finish_property()
        })
        .add_daylight_time(|b| {
            // RFC 5545: 3.8.2.4
            b.add_date_time_start(
                time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                Some(time::Time::from_hms(15, 0, 0).unwrap()),
            )
            .add_iana_param("date-time-start-test", "test")
            .add_x_param("x-date-time-start-test", "test")
            .finish_property()
            // RFC 5545: 3.8.3.4
            .add_time_zone_offset_to(TimeZoneOffset::new(1, 4, 0, None))
            .add_iana_param("time-zone-offset-to-test", "test")
            .add_x_param("x-time-zone-offset-to-test", "test")
            .finish_property()
            // RFC 5545: 3.8.3.3
            .add_time_zone_offset_from(TimeZoneOffset::new(-1, 4, 0, None))
            .add_iana_param("time-zone-offset-from-test", "test")
            .add_x_param("x-time-zone-offset-from-test", "test")
            .finish_property()
            // RFC 5545: 3.8.5.3
            .add_recurrence_rule(RecurFreq::Yearly, |rule| {
                // Likely a junk rule, it's just to check that everything is settable and round trips correctly
                rule.set_until(
                    (
                        time::Date::from_calendar_date(2035, time::Month::August, 20).unwrap(),
                        time::Time::from_hms(15, 0, 0).unwrap(),
                        true,
                    )
                        .into(),
                )
                .set_count(1_000)
                .set_interval(4)
                .set_by_second(vec![1, 2, 3])
                .set_by_minute(vec![1, 2, 3])
                .set_by_hour(vec![1, 2, 3])
                .set_by_day(vec![OffsetWeekday::new(Weekday::Monday, None)])
                .set_by_month_day(vec![1, 2, 3])
                .set_by_year_day(vec![1, 2, 3])
                .set_by_week_number(vec![1, 2, 3])
                .set_by_month(vec![
                    time::Month::January,
                    time::Month::February,
                    time::Month::March,
                ])
                .set_by_set_pos(vec![1, 2, 3])
                .set_week_start(Weekday::Monday)
            })
            .add_iana_param("recurrence-rule-test", "test")
            .add_x_param("x-recurrence-rule-test", "test")
            .finish_property()
            // RFC 5545: 3.8.1.4
            .add_comment("test")
            .add_alternate_representation("http://example.com/test")
            .add_language(LanguageTag::new("en").with_region("US"))
            .add_iana_param("comment-test", "test")
            .add_x_param("x-comment-test", "test")
            .finish_property()
            // RFC 5545: 3.8.5.2
            .add_recurrence_date_date_times(vec![
                (
                    time::Date::from_calendar_date(2024, time::Month::August, 8).unwrap(),
                    Some(time::Time::from_hms(15, 0, 0).unwrap()),
                    true,
                )
                    .into(),
                (
                    time::Date::from_calendar_date(2024, time::Month::August, 9).unwrap(),
                    Some(time::Time::from_hms(15, 0, 0).unwrap()),
                    true,
                )
                    .into(),
            ])
            .add_tz_id("test", true)
            .add_iana_param("recurrence-date-test", "test")
            .add_x_param("x-recurrence-date-test", "test")
            .finish_property()
            .add_time_zone_name("test")
            .add_iana_param("time-zone-name-test", "test")
            .add_x_param("x-time-zone-name-test", "test")
            .finish_property()
            // RFC 5545: 3.8.8.1
            .add_iana_property("other", "some-value")
            .add_iana_param("other-test", "test")
            .add_x_param("x-other-test", "test")
            .finish_property()
            // RFC 5545: 3.8.8.2
            .add_x_property("x-other", "some-value")
            .add_iana_param("x-other-test", "test")
            .add_x_param("x-x-other-test", "test")
            .finish_property()
        })
        .finish_component()
        .add_iana_component("other-comp", |c| {
            c.add_iana_property("other", "some-value")
                .add_iana_param("other-test", "test")
                .add_x_param("x-other-test", "test")
                .finish_property()
                .add_x_property("x-other", "some-value")
                .add_iana_param("other-test", "test")
                .add_x_param("x-other-test", "test")
                .finish_property()
        })
        .add_x_component("x-other-comp", |c| {
            c.add_iana_property("other", "some-value")
                .add_iana_param("other-test", "test")
                .add_x_param("x-other-test", "test")
                .finish_property()
                .add_x_property("x-other", "some-value")
                .add_iana_param("other-test", "test")
                .add_x_param("x-other-test", "test")
                .finish_property()
        })
        .build()
}
