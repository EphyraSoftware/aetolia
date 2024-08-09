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
                    tz.property_opt::<TimeZoneIdProperty>().unwrap().value().id
                );
            }
            CalendarComponent::Event(e) => {
                println!(
                    "Found event with description: {}",
                    e.property_opt::<DescriptionProperty>().unwrap().value()
                );

                let attendee = e.property_opt::<AttendeeProperty>().unwrap();
                let role_param = attendee.param_opt::<RoleParam>().unwrap();
                println!(
                    "Found attendee {} with role: {:?}",
                    attendee.value(),
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
