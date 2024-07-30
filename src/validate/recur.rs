use crate::common::RecurFreq;
use crate::model::{ComponentProperty, RecurRulePart, RecurrenceRule};
use crate::validate::{
    component_property_name, ComponentPropertyError, ComponentPropertyLocation,
    WithinPropertyLocation,
};
use std::collections::HashMap;

pub(super) fn validate_recurrence_rule(
    errors: &mut Vec<ComponentPropertyError>,
    property: &ComponentProperty,
    rule: &RecurrenceRule,
    property_index: usize,
) -> anyhow::Result<()> {
    if rule.parts.is_empty() {
        errors.push(ComponentPropertyError {
            message: "Recurrence rule is empty".to_string(),
            location: Some(ComponentPropertyLocation {
                index: property_index,
                name: component_property_name(property).to_string(),
                property_location: Some(WithinPropertyLocation::Value),
            }),
        });
        return Ok(());
    }

    let freq = match &rule.parts[0] {
        RecurRulePart::Freq(freq) => {
            // Frequency should be the first part, this is correct
            freq
        }
        _ => {
            errors.push(ComponentPropertyError {
                message: "Recurrence rule must start with a frequency".to_string(),
                location: Some(ComponentPropertyLocation {
                    index: property_index,
                    name: component_property_name(property).to_string(),
                    property_location: Some(WithinPropertyLocation::Value),
                }),
            });

            let maybe_freq = rule.parts.iter().find_map(|part| {
                if let RecurRulePart::Freq(freq) = part {
                    Some(freq)
                } else {
                    None
                }
            });

            match maybe_freq {
                Some(freq) => freq,
                None => {
                    errors.push(ComponentPropertyError {
                        message: "No frequency part found in recurrence rule, but it is required. This prevents the rest of the rule being checked".to_string(),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                    return Ok(());
                }
            }
        }
    };

    let mut seen_count = HashMap::<String, u32>::new();
    let add_count = |seen_count: &mut HashMap<String, u32>, key: &str| {
        *seen_count
            .entry(key.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1)
    };
    for (part_index, part) in rule.parts.iter().enumerate().skip(1) {
        match part {
            RecurRulePart::Freq(_) => {
                errors.push(ComponentPropertyError {
                    message: format!("Repeated FREQ part at index {part_index}"),
                    location: Some(ComponentPropertyLocation {
                        index: property_index,
                        name: component_property_name(property).to_string(),
                        property_location: Some(WithinPropertyLocation::Value),
                    }),
                });
            }
            RecurRulePart::Until(_) => {
                let count = add_count(&mut seen_count, "UNTIL");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated UNTIL part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The value of the UNTIL rule part MUST have the same value type as the "DTSTART" property.
            }
            RecurRulePart::Count(_) => {
                let count = add_count(&mut seen_count, "COUNT");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated COUNT part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::Interval(_) => {
                let count = add_count(&mut seen_count, "INTERVAL");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated INTERVAL part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::BySecList(second_list) => {
                let count = add_count(&mut seen_count, "BYSECOND");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYSECOND part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !second_list.iter().all(|second| *second <= 60) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYSECOND part at index {part_index}, seconds must be between 0 and 60"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified when the associated "DTSTART" property has a DATE value type.
            }
            RecurRulePart::ByMinute(minute_list) => {
                let count = add_count(&mut seen_count, "BYMINUTE");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYMINUTE part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !minute_list.iter().all(|minute| *minute <= 59) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYMINUTE part at index {part_index}, minutes must be between 0 and 59"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified when the associated "DTSTART" property has a DATE value type.
            }
            RecurRulePart::ByHour(hour_list) => {
                let count = add_count(&mut seen_count, "BYHOUR");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYHOUR part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !hour_list.iter().all(|hour| *hour <= 23) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYHOUR part at index {part_index}, hours must be between 0 and 23"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                // TODO The BYSECOND, BYMINUTE and BYHOUR rule parts MUST NOT be specified when the associated "DTSTART" property has a DATE value type.
            }
            RecurRulePart::ByDay(day_list) => {
                let count = add_count(&mut seen_count, "BYDAY");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYDAY part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                match freq {
                    RecurFreq::Monthly => {
                        // Offsets are permitted for this frequency
                    }
                    RecurFreq::Yearly => {
                        let is_by_week_number_specified = rule
                            .parts
                            .iter()
                            .any(|part| matches!(part, RecurRulePart::ByWeekNumber(_)));

                        if is_by_week_number_specified
                            && day_list.iter().any(|day| day.offset_weeks.is_some())
                        {
                            errors.push(ComponentPropertyError {
                                message: format!("BYDAY part at index {part_index} has a day with an offset, but the frequency is YEARLY and a BYWEEKNO part is specified"),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                    _ => {
                        if day_list.iter().any(|day| day.offset_weeks.is_some()) {
                            errors.push(ComponentPropertyError {
                                message: format!("BYDAY part at index {part_index} has a day with an offset, but the frequency is not MONTHLY or YEARLY"),
                                location: Some(ComponentPropertyLocation {
                                    index: property_index,
                                    name: component_property_name(property).to_string(),
                                    property_location: Some(WithinPropertyLocation::Value),
                                }),
                            });
                        }
                    }
                }
            }
            RecurRulePart::ByMonthDay(month_day_list) => {
                let count = add_count(&mut seen_count, "BYMONTHDAY");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYMONTHDAY part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !month_day_list
                    .iter()
                    .all(|day| (-31 <= *day && *day <= -1) || (1 <= *day && *day <= 31))
                {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYMONTHDAY part at index {part_index}, days must be between 1 and 31, or -31 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if freq == &RecurFreq::Weekly {
                    errors.push(ComponentPropertyError {
                        message: format!("BYMONTHDAY part at index {part_index} is not valid for a WEEKLY frequency"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::ByYearDay(year_day_list) => {
                let count = add_count(&mut seen_count, "BYYEARDAY");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYYEARDAY part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !year_day_list
                    .iter()
                    .all(|day| (-366 <= *day && *day <= -1) || (1 <= *day && *day <= 366))
                {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYYEARDAY part at index {part_index}, days must be between 1 and 366, or -366 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                match freq {
                    RecurFreq::Daily | RecurFreq::Weekly | RecurFreq::Monthly => {
                        errors.push(ComponentPropertyError {
                            message: format!("BYYEARDAY part at index {part_index} is not valid for a DAILY, WEEKLY or MONTHLY frequency"),
                            location: Some(ComponentPropertyLocation {
                                index: property_index,
                                name: component_property_name(property).to_string(),
                                property_location: Some(WithinPropertyLocation::Value),
                            }),
                        });
                    }
                    _ => {}
                }
            }
            RecurRulePart::ByWeekNumber(week_list) => {
                let count = add_count(&mut seen_count, "BYWEEKNO");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYWEEKNO part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !week_list
                    .iter()
                    .all(|week| (-53 <= *week && *week <= -1) || (1 <= *week && *week <= 53))
                {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYWEEKNO part at index {part_index}, weeks must be between 1 and 53, or -53 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if freq != &RecurFreq::Yearly {
                    errors.push(ComponentPropertyError {
                        message: format!("BYWEEKNO part at index {part_index} is only valid for a YEARLY frequency"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::ByMonth(_) => {
                let count = add_count(&mut seen_count, "BYMONTH");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYMONTH part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::WeekStart(_) => {
                let count = add_count(&mut seen_count, "WKST");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated WKST part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                let mut is_redundant = true;
                match freq {
                    RecurFreq::Weekly => {
                        let has_non_default_interval = rule.parts.iter().any(|part| matches!(part, RecurRulePart::Interval(interval) if *interval > 1));
                        let by_day_specified = rule
                            .parts
                            .iter()
                            .any(|part| matches!(part, RecurRulePart::ByDay(_)));
                        if has_non_default_interval && by_day_specified {
                            is_redundant = false;
                        }
                    }
                    RecurFreq::Yearly => {
                        let by_week_number_specified = rule
                            .parts
                            .iter()
                            .any(|part| matches!(part, RecurRulePart::ByWeekNumber(_)));
                        if by_week_number_specified {
                            is_redundant = false;
                        }
                    }
                    _ => {
                        // Otherwise, it's definitely redundant
                    }
                }

                if is_redundant {
                    errors.push(ComponentPropertyError {
                        message: format!("WKST part at index {part_index} is redundant"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
            RecurRulePart::BySetPos(set_pos_list) => {
                let count = add_count(&mut seen_count, "BYSETPOS");
                if count > 1 {
                    errors.push(ComponentPropertyError {
                        message: format!("Repeated BYSETPOS part at index {part_index}"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                if !set_pos_list.iter().all(|set_pos| {
                    (-366 <= *set_pos && *set_pos <= -1) || (1 <= *set_pos && *set_pos <= 366)
                }) {
                    errors.push(ComponentPropertyError {
                        message: format!("Invalid BYSETPOS part at index {part_index}, set positions must be between 1 and 366, or -366 and -1"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }

                let has_other_by_rule = rule.parts.iter().any(|part| {
                    matches!(
                        part,
                        RecurRulePart::BySecList(_)
                            | RecurRulePart::ByMinute(_)
                            | RecurRulePart::ByHour(_)
                            | RecurRulePart::ByDay(_)
                            | RecurRulePart::ByMonthDay(_)
                            | RecurRulePart::ByYearDay(_)
                            | RecurRulePart::ByWeekNumber(_)
                            | RecurRulePart::ByMonth(_)
                    )
                });
                if !has_other_by_rule {
                    errors.push(ComponentPropertyError {
                        message: format!("BYSETPOS part at index {part_index} is not valid without another BYxxx rule part"),
                        location: Some(ComponentPropertyLocation {
                            index: property_index,
                            name: component_property_name(property).to_string(),
                            property_location: Some(WithinPropertyLocation::Value),
                        }),
                    });
                }
            }
        }
    }

    Ok(())
}
