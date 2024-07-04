mod component;
mod param;
mod property;

trait ToModel {
    type Model;

    fn to_model(&self) -> anyhow::Result<Self::Model>;
}

impl<T> ToModel for Vec<T>
where
    T: ToModel,
{
    type Model = Vec<T::Model>;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        self.iter().map(ToModel::to_model).collect()
    }
}

impl ToModel for crate::parser::ICalendar<'_> {
    type Model = crate::model::ICalObject;

    fn to_model(&self) -> anyhow::Result<Self::Model> {
        todo!()
    }
}

fn convert_string(input: &[u8]) -> String {
    String::from_utf8_lossy(input).to_string()
}
