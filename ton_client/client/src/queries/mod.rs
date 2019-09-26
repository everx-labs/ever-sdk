use dispatch::DispatchTable;

pub(crate) mod query;

pub(crate) fn register(handlers: &mut DispatchTable) {
    handlers.spawn("queries.query",
        query::query);
    handlers.spawn("queries.wait.for",
        query::wait_for);
    handlers.spawn("queries.subscribe",
        query::subscribe);
    handlers.spawn("queries.get.next",
        query::get_next);
    handlers.spawn("queries.unsubscribe",
        query::unsubscribe);
}
