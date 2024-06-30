mod param;

trait ToModel {
    type Model;

    fn to_model(&self) -> Self::Model;
}

impl ToModel for crate::parser::ICalendar<'_> {
    type Model = crate::model::ICalObject;

    fn to_model(&self) -> Self::Model {
        todo!()
    }
}

fn convert_string(input: &[u8]) -> String {
    String::from_utf8_lossy(input).to_string()
}
