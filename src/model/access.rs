use crate::model::param::{Param, ParamInner};
use crate::model::property::{ComponentPropertiesInner, ComponentProperty, ComponentPropertyInner};

pub trait ComponentAccess {
    fn properties(&self) -> &[ComponentProperty];

    fn get_property<T>(&self) -> Option<&T>
    where
        ComponentProperty: ComponentPropertyInner<T>,
    {
        self.properties().iter().find_map(|p| p.property_inner())
    }

    fn get_properties<T>(&self) -> Vec<&T>
    where
        ComponentProperty: ComponentPropertiesInner<T>,
    {
        self.properties()
            .iter()
            .filter_map(|p| p.many_property_inner())
            .collect()
    }

    fn get_iana_properties(&self, name: &str) -> Vec<&str> {
        self.properties()
            .iter()
            .filter_map(|p| match p {
                ComponentProperty::IanaProperty(p) if p.name == name => Some(p.value.as_str()),
                _ => None,
            })
            .collect()
    }

    fn get_x_properties(&self, name: &str) -> Vec<&str> {
        self.properties()
            .iter()
            .filter_map(|p| match p {
                ComponentProperty::XProperty(p) if p.name == name => Some(p.value.as_str()),
                _ => None,
            })
            .collect()
    }
}

macro_rules! impl_component_access {
    ($for_type:ty) => {
        impl $crate::model::access::ComponentAccess for $for_type {
            fn properties(&self) -> &[$crate::model::property::ComponentProperty] {
                &self.properties
            }
        }
    };
}

pub(crate) use impl_component_access;

pub trait PropertyAccess<V> {
    fn value(&self) -> &V;

    fn params(&self) -> &[Param];

    fn get_param<T>(&self) -> Option<&T>
    where
        Param: ParamInner<T>,
    {
        self.params().iter().find_map(|p| p.param_inner())
    }

    fn get_iana_params(&self, name: &str) -> Vec<&str> {
        self.params()
            .iter()
            .filter_map(|p| match p {
                Param::Other {
                    name: param_name,
                    value,
                } if param_name == name => Some(value.as_str()),
                _ => None,
            })
            .collect()
    }

    fn get_x_params(&self, name: &str) -> Vec<&str> {
        self.params()
            .iter()
            .filter_map(|p| match p {
                Param::Other {
                    name: param_name,
                    value,
                } if param_name == name => Some(value.as_str()),
                _ => None,
            })
            .collect()
    }
}

macro_rules! impl_property_access {
    ($for_type:ty, $value_type:ty) => {
        impl $crate::model::access::PropertyAccess<$value_type> for $for_type {
            fn value(&self) -> &$value_type {
                &self.value
            }

            fn params(&self) -> &[$crate::model::param::Param] {
                &self.params
            }
        }
    };
}

pub(crate) use impl_property_access;
