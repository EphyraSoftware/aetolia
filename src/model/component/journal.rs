use crate::model::component::{
    add_attach, add_categories, add_class, add_comment, add_contact, add_created,
    add_date_time_stamp, add_date_time_start, add_description, add_exception_date_times,
    add_last_modified, add_organizer, add_recurrence_date, add_recurrence_id, add_recurrence_rule,
    add_related, add_request_status, add_sequence, add_summary, add_unique_identifier, add_url,
    impl_finish_component_build, impl_other_component_properties, CalendarComponent,
};
use crate::model::impl_component_access;
use crate::model::object::ICalObjectBuilder;
use crate::model::param::ParticipationStatusJournal;
use crate::model::property::{
    AddComponentProperty, AttendeePropertyBuilder, ComponentProperty, IanaComponentPropertyBuilder,
    StatusJournal, StatusPropertyBuilder, XComponentPropertyBuilder,
};

#[derive(Debug, PartialEq)]
pub struct JournalComponent {
    pub(crate) properties: Vec<ComponentProperty>,
}

impl_component_access!(JournalComponent);

impl JournalComponent {
    pub(crate) fn new() -> Self {
        JournalComponent {
            properties: Vec::new(),
        }
    }
}

impl Default for JournalComponent {
    fn default() -> Self {
        Self::new()
    }
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
        value: &str,
    ) -> AttendeePropertyBuilder<Self, ParticipationStatusJournal> {
        AttendeePropertyBuilder::new(self, value.to_string())
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
