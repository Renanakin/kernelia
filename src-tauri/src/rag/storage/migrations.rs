pub const INIT_SQL: &str = include_str!("../../../migrations/0001_kernelia_rag_init.sql");
pub const TICKETS_SQL: &str = include_str!("../../../migrations/0002_kernelia_rag_support_tickets.sql");
pub const BASE_SEED_SQL: &str = include_str!("../../../seeds/0001_kernelia_rag_seed_base.sql");
pub const TOOL_CATALOG_SEED_SQL: &str =
    include_str!("../../../seeds/0002_kernelia_rag_seed_tool_catalog.sql");
pub const COMMAND_CATALOG_SEED_SQL: &str =
    include_str!("../../../seeds/0003_kernelia_rag_seed_command_catalog.sql");

pub fn all_migrations() -> [(&'static str, &'static str); 2] {
    [
        ("0001_kernelia_rag_init.sql", INIT_SQL),
        ("0002_kernelia_rag_support_tickets.sql", TICKETS_SQL),
    ]
}

pub fn all_seeds() -> [(&'static str, &'static str); 3] {
    [
        ("0001_kernelia_rag_seed_base.sql", BASE_SEED_SQL),
        ("0002_kernelia_rag_seed_tool_catalog.sql", TOOL_CATALOG_SEED_SQL),
        ("0003_kernelia_rag_seed_command_catalog.sql", COMMAND_CATALOG_SEED_SQL),
    ]
}
