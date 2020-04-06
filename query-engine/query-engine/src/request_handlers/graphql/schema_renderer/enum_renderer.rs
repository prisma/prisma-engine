use super::*;

pub struct GqlEnumRenderer<'a> {
    enum_type: &'a EnumType,
}

impl<'a> Renderer for GqlEnumRenderer<'a> {
    fn render(&self, ctx: RenderContext) -> (String, RenderContext) {
        if ctx.already_rendered(self.enum_type.name()) {
            return ("".to_owned(), ctx);
        }

        let values = self.format_enum_values();
        let rendered = format!("enum {} {{\n{}\n}}", self.enum_type.name(), values.join("\n"));

        ctx.add(self.enum_type.name().to_owned(), rendered.clone());
        (rendered, ctx)
    }
}

impl<'a> GqlEnumRenderer<'a> {
    pub fn new(enum_type: &EnumType) -> GqlEnumRenderer {
        GqlEnumRenderer { enum_type }
    }

    fn format_enum_values(&self) -> Vec<String> {
        match self.enum_type {
            EnumType::Internal(i) => i.external_values(),
            EnumType::OrderBy(ord) => ord.values(),
        }
    }
}
