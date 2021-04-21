pub mod mssql;

mod capabilities;
mod features;
mod tags;

use std::collections::HashSet;

pub use capabilities::*;
pub use features::*;
pub use tags::*;

use enumflags2::BitFlags;
use once_cell::sync::Lazy;

static SKIP_CONNECTORS: Lazy<HashSet<String>> = Lazy::new(|| {
    std::env::var("SKIP_CONNECTORS")
        .map(|s| s.split(",").map(ToString::to_string).collect())
        .unwrap_or_else(|_| HashSet::new())
});

fn connector_names() -> Vec<(&'static str, BitFlags<Tags>)> {
    vec![
        ("mssql_2017", (Tags::Mssql | Tags::Mssql2017)),
        ("mssql_2019", (Tags::Mssql | Tags::Mssql2019)),
        ("mysql_8", Tags::Mysql | Tags::Mysql8),
        ("mysql_5_7", Tags::Mysql | Tags::Mysql57),
        ("mysql_5_6", Tags::Mysql | Tags::Mysql56),
        ("postgres9", Tags::Postgres.into()),
        ("postgres10", Tags::Postgres.into()),
        ("postgres11", Tags::Postgres.into()),
        ("postgres12", Tags::Postgres | Tags::Postgres12),
        ("postgres13", Tags::Postgres.into()),
        ("mysql_mariadb", Tags::Mysql | Tags::Mariadb),
        ("sqlite", Tags::Sqlite.into()),
        ("vitess_5_7", Tags::Mysql | Tags::Vitess57),
    ]
}

fn postgres_capabilities() -> BitFlags<Capabilities> {
    Capabilities::ScalarLists | Capabilities::Enums | Capabilities::Json | Capabilities::CreateDatabase
}

fn mysql_5_7_capabilities() -> BitFlags<Capabilities> {
    Capabilities::Enums | Capabilities::Json | Capabilities::CreateDatabase
}

fn mysql_5_6_capabilities() -> BitFlags<Capabilities> {
    Capabilities::Enums | Capabilities::CreateDatabase
}

fn mssql_2017_capabilities() -> BitFlags<Capabilities> {
    Capabilities::CreateDatabase.into()
}

fn mssql_2019_capabilities() -> BitFlags<Capabilities> {
    Capabilities::CreateDatabase.into()
}

fn vitess_5_7_capabilities() -> BitFlags<Capabilities> {
    Capabilities::Enums | Capabilities::Json
}

fn infer_capabilities(tags: BitFlags<Tags>) -> BitFlags<Capabilities> {
    if tags.intersects(Tags::Postgres) {
        return postgres_capabilities();
    }

    if tags.intersects(Tags::Mysql56) {
        return mysql_5_6_capabilities();
    }

    if tags.intersects(Tags::Mysql) {
        return mysql_5_7_capabilities();
    }

    if tags.intersects(Tags::Mssql2017) {
        return mssql_2017_capabilities();
    }

    if tags.intersects(Tags::Mssql2019) {
        return mssql_2019_capabilities();
    }

    if tags.intersects(Tags::Vitess57) {
        return vitess_5_7_capabilities();
    }

    BitFlags::empty()
}

pub static CONNECTORS: Lazy<Connectors> = Lazy::new(|| {
    let connectors: Vec<Connector> = connector_names()
        .iter()
        .filter(|(name, _)| !SKIP_CONNECTORS.contains(*name))
        .map(|(name, tags)| Connector {
            name: (*name).to_owned(),
            capabilities: infer_capabilities(*tags),
            tags: *tags,
        })
        .collect();

    Connectors::new(connectors)
});

pub struct Connectors {
    connectors: Vec<Connector>,
}

impl Connectors {
    fn new(connectors: Vec<Connector>) -> Connectors {
        Connectors { connectors }
    }

    pub fn all(&self) -> impl Iterator<Item = &Connector> {
        self.connectors.iter()
    }

    pub fn len(&self) -> usize {
        self.connectors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Represents a connector to be tested.
pub struct Connector {
    name: String,
    pub capabilities: BitFlags<Capabilities>,
    pub tags: BitFlags<Tags>,
}

impl Connector {
    /// The name of the connector.
    pub fn name(&self) -> &str {
        &self.name
    }
}
