use crate::parser::{
    CalendarScaleProperty, IanaProperty, MethodProperty, ProductIdProperty, VersionProperty,
    XProperty,
};

#[derive(Debug, Eq, PartialEq)]
pub enum CalendarProperty<'a> {
    ProductId(ProductIdProperty<'a>),
    Version(VersionProperty<'a>),
    CalendarScale(CalendarScaleProperty<'a>),
    Method(MethodProperty<'a>),
    XProperty(XProperty<'a>),
    IanaProperty(IanaProperty<'a>),
}
