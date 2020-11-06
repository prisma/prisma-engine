pub struct TypeNameValidator {
    reserved_names: Vec<&'static str>,
}

impl TypeNameValidator {
    pub fn new() -> Self {
        Self {
            reserved_names: vec![
                // The source of the following list is from prisma-client-js. Any edit should be done in both places.
                // https://github.com/prisma/prisma/blob/master/src/packages/client/src/generation/generateClient.ts#L443
                "dmmf",
                "PromiseType",
                "PromiseReturnType",
                "Enumerable",
                "MergeTruthyValues",
                "CleanupNever",
                "Subset",
                "AtLeastOne",
                "atMostOne",
                "OnlyOne",
                "PrismaClientFetcher",
                "PrismaClient",
                "Engine",
                "Datasources",
                "ErrorFormat",
                "Hooks",
                "LogLevel",
                "LogDefinition",
                "GetLogType",
                "GetEvents",
                "QueryEvent",
                "LogEvent",
                "ModelDelegate",
                "QueryDelegate",
                "missingArg",
                "ArgError",
                "InvalidFieldError",
                "InvalidFieldNameError",
                "InvalidFieldTypeError",
                "EmptySelectError",
                "NoTrueSelectError",
                "IncludeAndSelectError",
                "EmptyIncludeError",
                "InvalidArgError",
                "InvalidArgNameError",
                "MissingArgError",
                "InvalidArgTypeError",
                "AtLeastOneError",
                "AtMostOneError",
                "PrismaClientRequestError",
                "PrismaClientOptions",
                "PrismaClientKnownRequestError",
                "PrismaClientUnknownRequestError",
                "PrismaClientInitializationError",
                "PrismaClientRustPanicError",
                "PrismaVersion",
                "QueryMode",
                "SortOrder",
                "InputJsonValue",
                // JavaScript keywords
                "await",
                "async",
                "break",
                "case",
                "catch",
                "class",
                "const",
                "continue",
                "debugger",
                "default",
                "delete",
                "do",
                "else",
                "enum",
                "export",
                "extends",
                "false",
                "finally",
                "for",
                "function",
                "if",
                "implements",
                "import",
                "in",
                "instanceof",
                "interface",
                "let",
                "new",
                "null",
                "package",
                "private",
                "protected",
                "public",
                "return",
                "super",
                "switch",
                "this",
                "throw",
                "true",
                "try",
                "typeof",
                "var",
                "void",
                "while",
                "with",
                "yield",
            ],
        }
    }

    pub fn is_reserved<T>(&self, name: T) -> bool
    where
        T: AsRef<str>,
    {
        self.reserved_names.contains(&name.as_ref())
    }
}
