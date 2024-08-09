use crate::model::{ComponentProperty, ComponentPropertyInner, Param, ParamInner};

pub trait ComponentAccess {
    fn properties(&self) -> &[ComponentProperty];

    fn property_opt<T>(&self) -> Option<&T>
    where
        ComponentProperty: ComponentPropertyInner<T>,
    {
        self.properties().iter().find_map(|p| p.property_inner())
    }

    fn iana_property_opt(&self, name: &str) -> Option<&str> {
        self.properties().iter().find_map(|p| match p {
            ComponentProperty::IanaProperty(p) if p.name == name => Some(p.value.as_str()),
            _ => None,
        })
    }

    fn x_property_opt(&self, name: &str) -> Option<&str> {
        self.properties().iter().find_map(|p| match p {
            ComponentProperty::XProperty(p) if p.name == name => Some(p.value.as_str()),
            _ => None,
        })
    }
}

macro_rules! impl_component_access {
    ($for_type:ty) => {
        impl $crate::model::ComponentAccess for $for_type {
            fn properties(&self) -> &[$crate::model::ComponentProperty] {
                &self.properties
            }
        }
    };
}

pub(crate) use impl_component_access;

pub trait PropertyAccess<V> {
    fn value(&self) -> &V;

    fn params(&self) -> &[Param];

    fn param_opt<T>(&self) -> Option<&T>
    where
        Param: ParamInner<T>,
    {
        self.params().iter().find_map(|p| p.param_inner())
    }
}

macro_rules! impl_property_access {
    ($for_type:ty, $value_type:ty) => {
        impl $crate::model::PropertyAccess<$value_type> for $for_type {
            fn value(&self) -> &$value_type {
                &self.value
            }

            fn params(&self) -> &[$crate::model::Param] {
                &self.params
            }
        }
    };
}

pub(crate) use impl_property_access;
