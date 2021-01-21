pub mod inputs {
    pub mod args {
        pub const WHERE: &str = "where";
        pub const DATA: &str = "data";

        // nested operations
        pub const CREATE: &str = "create";
        pub const CONNECT_OR_CREATE: &str = "connectOrCreate";
        pub const CONNECT: &str = "connect";
        pub const DISCONNECT: &str = "disconnect";
        pub const UPDATE: &str = "update";
        pub const UPDATE_MANY: &str = "updateMany";
        pub const UPSERT: &str = "upsert";
        pub const DELETE: &str = "delete";
        pub const DELETE_MANY: &str = "deleteMany";
        // scalar lists
        pub const SET: &str = "set";
        // numbers
        pub const INCREMENT: &str = "increment";
        pub const DECREMENT: &str = "decrement";
        pub const MULTIPLY: &str = "multiply";
        pub const DIVIDE: &str = "divide";

        // pagination args
        pub const CURSOR: &str = "cursor";
        pub const TAKE: &str = "take";
        pub const SKIP: &str = "skip";

        // sorting args
        pub const ORDER_BY: &str = "orderBy";

        // aggregation args
        pub const BY: &str = "by";
        pub const HAVING: &str = "having";

        // raw specific args
        pub const QUERY: &str = "query";
        pub const PARAMETERS: &str = "parameters";

        pub const DISTINCT: &str = "distinct";
    }
    pub mod filters {
        // scalar filters
        pub const EQUALS: &str = "equals";
        pub const CONTAINS: &str = "contains";
        pub const STARTS_WITH: &str = "startsWith";
        pub const ENDS_WITH: &str = "endsWith";
        pub const LOWER_THAN: &str = "lt";
        pub const LOWER_THAN_OR_EQUAL: &str = "lte";
        pub const GREATER_THAN: &str = "gt";
        pub const GREATER_THAN_OR_EQUAL: &str = "gte";
        pub const IN: &str = "in";

        // legacy filter
        pub const NOT_IN: &str = "notIn";

        // case-sensitivity filters
        pub const MODE: &str = "mode";
        pub const INSENSITIVE: &str = "insensitive";
        pub const DEFAULT: &str = "default";

        // condition filters
        pub const AND: &str = "AND";
        pub const AND_LOWERCASE: &str = "and";
        pub const OR: &str = "OR";
        pub const OR_LOWERCASE: &str = "or";
        pub const NOT: &str = "NOT";
        pub const NOT_LOWERCASE: &str = "not";

        // List-specific filters
        pub const HAS: &str = "has";
        pub const HAS_NONE: &str = "hasNone";
        pub const HAS_SOME: &str = "hasSome";
        pub const HAS_EVERY: &str = "hasEvery";
        pub const IS_EMPTY: &str = "isEmpty";

        // m2m filters
        pub const EVERY: &str = "every";
        pub const SOME: &str = "some";
        pub const NONE: &str = "none";

        // o2m filters
        pub const IS: &str = "is";
        pub const IS_NOT: &str = "isNot";

        // aggregation filters
        pub const COUNT: &str = "count";
        pub const AVG: &str = "avg";
        pub const SUM: &str = "sum";
        pub const MIN: &str = "min";
        pub const MAX: &str = "max";

        // ordering
        pub const ASC: &str = "asc";
        pub const DESC: &str = "desc";
    }
}

pub mod outputs {
    pub mod fields {
        // aggregation fields
        pub const COUNT: &str = "count";
        pub const AVG: &str = "avg";
        pub const MIN: &str = "min";
        pub const MAX: &str = "max";
        pub const SUM: &str = "sum";
    }
}
