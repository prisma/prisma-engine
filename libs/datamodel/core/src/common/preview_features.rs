// datasource preview features
const NATIVE_TYPES: &'static str = "nativeTypes";

// generator preview features
const ATOMIC_NUMBER_OPERATIONS: &'static str = "atomicNumberOperations";
const CONNECT_OR_CREATE: &'static str = "connectOrCreate";
const TRANSACTION_API: &'static str = "transactionApi";

// deprecated preview features
const AGGREGATE_API: &'static str = "aggregateApi"; // todo move this to deprecated preview features list for VSCode
const MIDDLEWARES: &'static str = "middlewares";
const DISTINCT: &'static str = "distinct";

pub const DATASOURCE_PREVIEW_FEATURES: [&'static str; 1] = [NATIVE_TYPES];
pub const GENERATOR_PREVIEW_FEATURES: [&'static str; 6] = [
    ATOMIC_NUMBER_OPERATIONS,
    CONNECT_OR_CREATE,
    TRANSACTION_API,
    AGGREGATE_API,
    MIDDLEWARES,
    DISTINCT,
];
