use super::directive::{
    new_builtin_enum_directives, new_builtin_enum_value_directives, new_builtin_field_directives,
    new_builtin_model_directives, DirectiveListValidator,
};
use crate::{configuration, dml};

pub struct DirectiveBox {
    pub field: DirectiveListValidator<dml::Field>,
    pub model: DirectiveListValidator<dml::Model>,
    pub enm: DirectiveListValidator<dml::Enum>,
    pub enm_value: DirectiveListValidator<dml::EnumValue>,
}

impl DirectiveBox {
    /// Creates a new instance, with all builtin directives registered.
    pub fn new() -> DirectiveBox {
        DirectiveBox {
            field: new_builtin_field_directives(),
            model: new_builtin_model_directives(),
            enm: new_builtin_enum_directives(),
            enm_value: new_builtin_enum_value_directives(),
        }
    }

    /// Creates a new instance, with all builtin directives and
    /// the directives defined by the given sources registered.
    ///
    /// The directives defined by the given sources will be namespaced.
    pub fn with_sources(_sources: &[Box<dyn configuration::Source + Send + Sync>]) -> DirectiveBox {
        //        sources.iter().fold(DirectiveBox::new(), |mut directives, source| {
        //            //            directives
        //            //                .enm
        //            //                .add_all_scoped(source.get_enum_directives(), source.name());
        //            //            directives
        //            //                .field
        //            //                .add_all_scoped(source.get_field_directives(), source.name());
        //            //            directives
        //            //                .model
        //            //                .add_all_scoped(source.get_model_directives(), source.name());
        //
        //            directives
        //        })
        DirectiveBox::new()
    }
}
