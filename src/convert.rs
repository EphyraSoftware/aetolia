use crate::error::AetoliaResult;

mod component;
mod object;
mod param;
mod property;

/// Conversion trait for converting parser model types to model types.
pub trait ToModel {
    type Model;

    fn to_model(&self) -> AetoliaResult<Self::Model>;
}

impl<T> ToModel for Vec<T>
where
    T: ToModel,
{
    type Model = Vec<T::Model>;

    fn to_model(&self) -> AetoliaResult<Self::Model> {
        self.iter().map(ToModel::to_model).collect()
    }
}

fn convert_string(input: &[u8]) -> String {
    String::from_utf8_lossy(input).to_string()
}
