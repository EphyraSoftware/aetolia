use crate::model::{add_attach, add_categories, add_class, add_comment, add_contact, add_created, add_date_time_stamp, add_date_time_start, add_description, add_duration, add_exception_date_times, add_geographic_position, add_last_modified, add_location, add_organizer, add_priority, add_recurrence_date, add_recurrence_id, add_recurrence_rule, add_related, add_request_status, add_resources, add_sequence, add_summary, add_unique_identifier, add_url, AddComponentProperty, AttendeeParamBuilder, CalendarComponent, CompletedPropertyBuilder, ComponentProperty, DueDateTimePropertyBuilder, IanaComponentPropertyBuilder, ICalObjectBuilder, impl_finish_component_build, impl_other_component_properties, ParticipationStatusJournal, ParticipationStatusToDo, PercentCompletePropertyBuilder, StatusJournal, StatusPropertyBuilder, StatusToDo, XComponentPropertyBuilder};
use crate::model::component::todo::{ToDoComponent, ToDoComponentBuilder};

pub struct JournalComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

pub struct JournalComponentBuilder {
    owner: ICalObjectBuilder,
    inner: JournalComponent,
}

impl JournalComponentBuilder {
    pub(crate) fn new(owner: ICalObjectBuilder) -> Self {
        JournalComponentBuilder {
            owner,
            inner: JournalComponent {
                properties: Vec::new(),
            },
        }
    }

    add_date_time_stamp!();

    add_unique_identifier!();

    add_class!();

    add_created!();

    add_date_time_start!();

    add_last_modified!();

    add_organizer!();

    add_recurrence_id!();

    add_sequence!();

    pub fn add_status(self, value: StatusJournal) -> StatusPropertyBuilder<Self> {
        StatusPropertyBuilder::new(self, value.into())
    }

    add_summary!();

    add_url!();

    add_recurrence_rule!();

    add_attach!();

    pub fn add_attendee(
        self,
        value: String,
    ) -> AttendeeParamBuilder<Self, ParticipationStatusJournal> {
        AttendeeParamBuilder::new(self, value)
    }

    add_categories!();

    add_comment!();

    add_contact!();

    add_description!();

    add_exception_date_times!();

    add_related!();

    add_recurrence_date!();

    add_request_status!();

    impl_other_component_properties!(
        XComponentPropertyBuilder,
        IanaComponentPropertyBuilder,
        JournalComponentBuilder
    );

    impl_finish_component_build!(CalendarComponent::Journal);
}

impl AddComponentProperty for JournalComponentBuilder {
    fn add_property(&mut self, property: ComponentProperty) {
        self.inner.properties.push(property);
    }
}
