mod string_builder;
mod table;

use crate::ast;

pub use string_builder::StringBuilder;
pub use table::TableFormat;

pub trait LineWriteable {
    fn write(&mut self, param: &str);
    fn line_empty(&self) -> bool;
    fn end_line(&mut self);
    fn maybe_end_line(&mut self);
}

pub struct Renderer<'a> {
    stream: &'a mut dyn std::io::Write,
    indent: usize,
    new_line: usize,
    is_new: bool,
    maybe_new_line: usize,
    indent_width: usize,
}

// TODO: It would be soooo cool if we could pass format strings around.
impl<'a> Renderer<'a> {
    pub fn new(stream: &'a mut dyn std::io::Write, indent_width: usize) -> Renderer<'a> {
        Renderer {
            stream,
            indent: 0,
            indent_width,
            new_line: 0,
            maybe_new_line: 0,
            is_new: true,
        }
    }

    pub fn render(&mut self, datamodel: &ast::SchemaAst) {
        let mut type_renderer: Option<TableFormat> = None;

        for (i, top) in datamodel.tops.iter().enumerate() {
            match &top {
                // TODO: This is super ugly. Goal is that type groups get formatted together.
                ast::Top::Type(custom_type) => {
                    if type_renderer.is_none() {
                        if i != 0 {
                            // We put an extra line break in between top level structs.
                            self.end_line();
                        }
                        type_renderer = Some(TableFormat::new());
                    }
                    if let Some(renderer) = &mut type_renderer {
                        Self::render_custom_type(renderer, custom_type);
                    }
                }
                other => {
                    if let Some(renderer) = &type_renderer {
                        renderer.render(self);
                        type_renderer = None;
                    }

                    if i != 0 {
                        // We put an extra line break in between top level structs.
                        self.end_line();
                    }

                    match other {
                        ast::Top::Model(model) => self.render_model(model),
                        ast::Top::Enum(enm) => self.render_enum(enm),
                        ast::Top::Source(source) => self.render_source_block(source),
                        ast::Top::Generator(generator) => self.render_generator_block(generator),
                        ast::Top::Type(_) => unreachable!(),
                    }
                }
            };
        }
    }

    fn render_documentation(target: &mut dyn LineWriteable, obj: &dyn ast::WithDocumentation) {
        if let Some(doc) = &obj.documentation() {
            for line in doc.text.split('\n') {
                target.write("/// ");
                target.write(line);
                target.end_line();
            }
        }
    }

    fn render_source_block(&mut self, source: &ast::SourceConfig) {
        Self::render_documentation(self, source);

        self.write("datasource ");
        self.write(&source.name.name);
        self.write(" {");
        self.end_line();
        self.indent_up();

        let mut formatter = TableFormat::new();

        for property in &source.properties {
            formatter.write(&property.name.name);
            formatter.write(" = ");
            formatter.write(&Self::render_value_to_string(&property.value));
            formatter.end_line();
        }

        formatter.render(self);

        self.indent_down();
        self.write("}");
        self.end_line();
    }

    fn render_generator_block(&mut self, generator: &ast::GeneratorConfig) {
        Self::render_documentation(self, generator);

        self.write("generator ");
        self.write(&generator.name.name);
        self.write(" {");
        self.end_line();
        self.indent_up();

        let mut formatter = TableFormat::new();

        for property in &generator.properties {
            formatter.write(&property.name.name);
            formatter.write(" = ");
            formatter.write(&Self::render_value_to_string(&property.value));
            formatter.end_line();
        }

        formatter.render(self);

        self.indent_down();
        self.write("}");
        self.end_line();
    }

    fn render_custom_type(target: &mut TableFormat, field: &ast::Field) {
        Self::render_documentation(&mut target.interleave_writer(), field);

        target.write("type ");
        target.write(&field.name.name);
        target.write(&" = ");
        target.write(&field.field_type.name);

        // Attributes
        if !field.directives.is_empty() {
            let mut attributes_builder = StringBuilder::new();

            for directive in &field.directives {
                Self::render_field_directive(&mut attributes_builder, &directive);
            }

            target.write(&attributes_builder.to_string());
        }

        target.end_line();
    }

    fn render_model(&mut self, model: &ast::Model) {
        let comment_out = if model.commented_out {
            "// ".to_string()
        } else {
            "".to_string()
        };

        Self::render_documentation(self, model);

        self.write(format!("{}model ", comment_out).as_ref());
        self.write(&model.name.name);
        self.write(" {");
        self.end_line();
        self.indent_up();

        let mut field_formatter = TableFormat::new();

        for field in &model.fields {
            Self::render_field(&mut field_formatter, &field, model.commented_out);
        }

        field_formatter.render(self);

        if !model.directives.is_empty() {
            self.end_line();
            for directive in &model.directives {
                self.render_block_directive(&directive, comment_out.clone());
            }
        }

        self.indent_down();
        self.write(format!("{}{}", comment_out.clone(), "}").as_ref());
        self.end_line();
    }

    fn render_enum(&mut self, enm: &ast::Enum) {
        Self::render_documentation(self, enm);

        self.write("enum ");
        self.write(&enm.name.name);
        self.write(" {");
        self.end_line();
        self.indent_up();

        for value in &enm.values {
            self.write(&value.name.name);
            if !value.directives.is_empty() {
                let mut attributes_builder = StringBuilder::new();

                for directive in &value.directives {
                    attributes_builder.write(&" ");
                    Self::render_field_directive(&mut attributes_builder, &directive);
                }

                self.write(&attributes_builder.to_string());
            }

            self.end_line();
        }

        if !enm.directives.is_empty() {
            self.end_line();
            for directive in &enm.directives {
                self.write(" ");
                self.render_block_directive(&directive, "".to_string());
            }
        }

        self.indent_down();
        self.write("}");
        self.end_line();
    }

    fn render_field(target: &mut TableFormat, field: &ast::Field, is_commented_out: bool) {
        Self::render_documentation(&mut target.interleave_writer(), field);

        let commented_out = if field.is_commented_out || is_commented_out {
            "// ".to_string()
        } else {
            "".to_string()
        };

        target.write(format!("{}{}", &commented_out, &field.name.name).as_ref());

        // Type
        {
            let mut type_builder = StringBuilder::new();

            type_builder.write(&field.field_type.name);
            Self::render_field_arity(&mut type_builder, &field.arity);

            target.write(&type_builder.to_string());
        }

        // Attributes
        if !field.directives.is_empty() {
            let mut attributes_builder = StringBuilder::new();

            for directive in &field.directives {
                attributes_builder.write(&" ");
                Self::render_field_directive(&mut attributes_builder, &directive);
            }

            target.write(&attributes_builder.to_string());
        }

        target.end_line();
    }

    fn render_field_arity(target: &mut dyn LineWriteable, field_arity: &ast::FieldArity) {
        match field_arity {
            ast::FieldArity::List => target.write("[]"),
            ast::FieldArity::Optional => target.write("?"),
            ast::FieldArity::Required => {}
        };
    }

    fn render_field_directive(target: &mut dyn LineWriteable, directive: &ast::Directive) {
        target.write("@");
        target.write(&directive.name.name);

        if !directive.arguments.is_empty() {
            target.write("(");
            Self::render_arguments(target, &directive.arguments);
            target.write(")");
        }
    }

    fn render_block_directive(&mut self, directive: &ast::Directive, commented_out: String) {
        self.write(format!("{}@@", commented_out).as_ref());
        self.write(&directive.name.name);

        if !directive.arguments.is_empty() {
            self.write("(");
            Self::render_arguments(self, &directive.arguments);
            self.write(")");
        }

        self.end_line();
    }

    fn render_arguments(target: &mut dyn LineWriteable, args: &[ast::Argument]) {
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                target.write(&", ");
            }
            Self::render_argument(target, arg);
        }
    }

    fn render_argument(target: &mut dyn LineWriteable, args: &ast::Argument) {
        if args.name.name != "" {
            target.write(&args.name.name);
            target.write(&": ");
        }

        Self::render_value(target, &args.value);
    }

    pub(crate) fn render_value_to_string(val: &ast::Expression) -> String {
        let mut builder = StringBuilder::new();
        Self::render_value(&mut builder, val);
        builder.to_string()
    }

    fn render_value(target: &mut dyn LineWriteable, val: &ast::Expression) {
        match val {
            ast::Expression::Array(vals, _) => Self::render_array(target, &vals),
            ast::Expression::BooleanValue(val, _) => target.write(&val),
            ast::Expression::ConstantValue(val, _) => target.write(&val),
            ast::Expression::NumericValue(val, _) => target.write(&val),
            ast::Expression::StringValue(val, _) => Self::render_str(target, &val),
            ast::Expression::Function(name, args, _) => Self::render_func(target, &name, &args),
            ast::Expression::Any(_, _) => unimplemented!("Value of 'Any' type cannot be rendered."),
        };
    }

    fn render_func(target: &mut dyn LineWriteable, name: &str, vals: &[ast::Expression]) {
        target.write(name);
        target.write("(");
        for (idx, val) in vals.iter().enumerate() {
            if idx > 0 {
                target.write(", ");
            }

            Self::render_value(target, val);
        }
        target.write(")");
    }

    pub fn indent_up(&mut self) {
        self.indent += 1
    }

    pub fn indent_down(&mut self) {
        if self.indent == 0 {
            panic!("Indentation error.")
        }
        self.indent -= 1
    }

    fn render_array(target: &mut dyn LineWriteable, vals: &[ast::Expression]) {
        target.write(&"[");
        for (idx, arg) in vals.iter().enumerate() {
            if idx > 0 {
                target.write(&", ");
            }
            Self::render_value(target, arg);
        }
        target.write(&"]");
    }

    fn render_str(target: &mut dyn LineWriteable, param: &str) {
        target.write("\"");
        target.write(param);
        target.write("\"");
    }
}

impl<'a> LineWriteable for Renderer<'a> {
    fn write(&mut self, param: &str) {
        self.is_new = false;
        // TODO: Proper result handling.
        if self.new_line > 0 || self.maybe_new_line > 0 {
            for _i in 0..std::cmp::max(self.new_line, self.maybe_new_line) {
                writeln!(self.stream).expect("Writer error.");
            }
            write!(self.stream, "{}", " ".repeat(self.indent * self.indent_width)).expect("Writer error.");
            self.new_line = 0;
            self.maybe_new_line = 0;
        }

        write!(self.stream, "{}", param).expect("Writer error.");
    }

    fn end_line(&mut self) {
        self.new_line += 1;
    }

    fn maybe_end_line(&mut self) {
        self.maybe_new_line += 1;
    }

    fn line_empty(&self) -> bool {
        self.new_line != 0 || self.maybe_new_line != 0 || self.is_new
    }
}
