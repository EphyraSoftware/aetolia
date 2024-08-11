use aetolia::prelude::*;

fn main() {
    let file_path = std::env::args()
        .nth(1)
        .expect("No file provided, should be the first argument");

    let file = std::fs::File::open(&file_path).expect("Could not open file");
    let calendar = load_ical(file).expect("Failed to load iCalendar data");

    for (index, object) in calendar.iter().enumerate() {
        let errors = validate_model(object).expect("Failed to validate iCalendar data");
        println!("Ran validation on object at index: {:?}", index);

        for error in &errors {
            if error.severity == ICalendarErrorSeverity::Warning {
                println!("Warning: {}", error);
            } else {
                println!("Error: {}", error);
            }
        }

        if errors
            .iter()
            .filter(|e| e.severity == ICalendarErrorSeverity::Error)
            .count()
            == 0
        {
            println!("Object is valid");
        }
    }
}
