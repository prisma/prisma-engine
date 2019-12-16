use super::*;
use prisma_models::{EnumType, EnumValue};

pub struct DMMFEnumRenderer<'a> {
    enum_type: &'a EnumType,
}

impl<'a> Renderer<'a, ()> for DMMFEnumRenderer<'a> {
    fn render(&self, ctx: RenderContext) -> ((), RenderContext) {
        if ctx.already_rendered(&self.enum_type.name) {
            return ((), ctx);
        }

        let values: Vec<String> = self
            .enum_type
            .values
            .iter()
            .map(|v| self.format_enum_value(v))
            .collect();

        let rendered = DMMFEnum {
            name: self.enum_type.name.clone(),
            values,
        };

        ctx.add_enum(self.enum_type.name.clone(), rendered);
        ((), ctx)
    }
}

impl<'a> DMMFEnumRenderer<'a> {
    pub fn new(enum_type: &'a EnumType) -> DMMFEnumRenderer<'a> {
        DMMFEnumRenderer { enum_type }
    }

    fn format_enum_value(&self, value: &EnumValue) -> String {
        value.as_string().into_owned()
    }
}
