use aetolia::prelude::*;

fn main() {
    let input = std::fs::File::open("sample.ics").unwrap();
    let ical = load_ical(input).unwrap();
    println!("Loaded iCal document with {} object", ical.len());

    for component in &ical[0].components {
        match component {
            CalendarComponent::TimeZone(tz) => {
                println!(
                    "Found timezone with name: {}",
                    tz.get_property::<TimeZoneIdProperty>().unwrap().value().id
                );
            }
            CalendarComponent::Event(e) => {
                println!(
                    "Found event with description: {}",
                    e.get_property::<DescriptionProperty>().unwrap().value()
                );

                let attendees = e.get_properties::<AttendeeProperty>();
                let role_param = attendees[0].get_param::<RoleParam>().unwrap();
                println!(
                    "Found attendee {} with role: {:?}",
                    attendees[0].value(),
                    role_param.role
                );
            }
            _ => {}
        }
    }

    let validation_errors = validate_model(&ical[0]).unwrap();
    assert!(
        validation_errors.is_empty(),
        "Didn't expect any validation errors"
    );
}
