use std::collections::BTreeSet;
use toml_edit::visit::*;
use toml_edit::visit_mut::*;
use toml_edit::{Array, InlineTable, Item, KeyMut, Table, Value};

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[derive(Debug, Default)]
pub struct DebugVisitor {
    pub depth: usize,
}

impl<'doc> Visit<'doc> for DebugVisitor {
    fn visit_item(&mut self, node: &'doc Item) {
        let item_name = item_name(node);

        debug!("{}{item_name}: {}", "  ".repeat(self.depth), node);
        self.depth += 1;
        visit_item(self, node);
        self.depth -= 1;
    }

    // fn visit_value(&mut self, node: &'doc Value) {
    //     debug!("{}value: {}", "  ".repeat(self.depth), node);
    //     visit_value(self, node);
    // }

    fn visit_table_like(&mut self, node: &'doc dyn toml_edit::TableLike) {
        let repr = node
            .get_values()
            .iter()
            .map(|(k, v)| format!("{:?}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        debug!("{}table-like: {:?}", "  ".repeat(self.depth), repr);
        visit_table_like(self, node);
    }

    fn visit_table_like_kv(&mut self, key: &'doc str, node: &'doc Item) {
        debug!(
            "{}table-like-kv: {}: {}",
            "  ".repeat(self.depth),
            key,
            node,
        );
        visit_table_like_kv(self, key, node);
    }
}

fn item_name(node: &Item) -> &str {
    match node {
        Item::None => "none",
        Item::Value(val) => match val {
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Boolean(_) => "boolean",
            Value::Datetime(_) => "datetime",
            Value::Array(_) => "array",
            Value::InlineTable(_) => "inline-table",
        },
        Item::Table(table) if table.is_implicit() => "table-implicit",
        Item::Table(_table) => "table",
        Item::ArrayOfTables(_at) => "array-of-tables",
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VisitState {
    /// Represents the root of the table.
    Root,
    /// Represents "dependencies", "build-dependencies" or "dev-dependencies", or the target
    /// forms of these.
    Dependencies,
    /// A table within dependencies.
    SubDependencies,
    /// Represents "target".
    Target,
    /// "target.[TARGET]".
    TargetWithSpec,
    /// Represents some other state.
    Other,
}

impl VisitState {
    /// Figures out the next visit state, given the current state and the given key.
    fn descend(self, key: &str) -> Self {
        match (self, key) {
            (
                VisitState::Root | VisitState::TargetWithSpec,
                "dependencies" | "build-dependencies" | "dev-dependencies",
            ) => {
                debug!("Descend: {key}");
                VisitState::Dependencies
            }
            (VisitState::Root, "target") => VisitState::Target,
            (VisitState::Root | VisitState::TargetWithSpec, _) => VisitState::Other,
            (VisitState::Target, _) => VisitState::TargetWithSpec,
            (VisitState::Dependencies, _) => VisitState::SubDependencies,
            (VisitState::SubDependencies, _) => VisitState::SubDependencies,
            (VisitState::Other, _) => VisitState::Other,
        }
    }
}

/// Collect the names of every dependency key.
#[derive(Debug)]
pub struct DependencyNameVisitor<'doc> {
    pub state: VisitState,
    pub names: BTreeSet<&'doc str>,
}

impl<'doc> Visit<'doc> for DependencyNameVisitor<'doc> {
    fn visit_table_like_kv(&mut self, key: &'doc str, node: &'doc Item) {
        if self.state == VisitState::SubDependencies {
            debug!("subdep key: {key:?}");
        }

        if self.state == VisitState::Dependencies {
            debug!("getting key: {key:?}");
            self.names.insert(key);
        }

        // Since we're only interested in collecting the top-level keys right under
        // [dependencies], don't recurse unconditionally.

        let old_state = self.state;

        // Figure out the next state given the key.
        self.state = self.state.descend(key);

        // Recurse further into the document tree.
        visit_table_like_kv(self, key, node);

        // Restore the old state after it's done.
        self.state = old_state;
    }

    fn visit_table(&mut self, node: &'doc Table) {
        if self.state == VisitState::Dependencies {
            debug!("Visiting table: {:?}", node);
        }
        visit_table(self, node);
    }

    fn visit_inline_table(&mut self, node: &'doc InlineTable) {
        if self.state == VisitState::Dependencies {
            debug!("Visiting inline table: {:?}", node);
        }

        visit_inline_table(self, node)
    }

    fn visit_table_like(&mut self, node: &'doc dyn toml_edit::TableLike) {
        if self.state == VisitState::Dependencies {
            debug!("Visiting table-like");
        }

        visit_table_like(self, node);
    }

    fn visit_array(&mut self, node: &'doc Array) {
        if self.state == VisitState::Dependencies {
            debug!("Visiting array: {:?}", node);
        }
        visit_array(self, node);
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[derive(Debug)]
pub struct FeatureAddVisitor {
    pub state: VisitState,
    pub feature: String,
    pub dep: String,
    pub kind: String,
}

impl VisitMut for FeatureAddVisitor {
    fn visit_table_mut(&mut self, node: &mut Table) {
        visit_table_mut(self, node);

        // The conversion from regular tables into inline ones might leave some explicit parent
        // tables hanging, so convert them to implicit.
        if matches!(self.state, VisitState::Target | VisitState::TargetWithSpec) {
            // node.set_implicit(true);
        }
    }

    fn visit_table_like_kv_mut(&mut self, mut key: KeyMut<'_>, node: &mut Item) {
        let old_state = self.state;

        // Figure out the next state given the key.
        self.state = self.state.descend(key.get());

        match self.state {
            VisitState::Target | VisitState::TargetWithSpec | VisitState::Dependencies => {
                // Top-level dependency row, or above: turn inline tables into regular ones.
                if let Item::Value(Value::InlineTable(inline_table)) = node {
                    let inline_table = std::mem::replace(inline_table, InlineTable::new());
                    let table = inline_table.into_table();
                    key.fmt();
                    *node = Item::Table(table);
                }
            }
            VisitState::SubDependencies => {
                // Individual dependency: turn regular tables into inline ones.
                if let Item::Table(table) = node {
                    // Turn the table into an inline table.
                    let table = std::mem::replace(table, Table::new());
                    let inline_table = table.into_inline_table();
                    key.fmt();
                    *node = Item::Value(Value::InlineTable(inline_table));
                }
            }
            _ => {}
        }

        // Recurse further into the document tree.
        visit_table_like_kv_mut(self, key, node);

        // Restore the old state after it's done.
        self.state = old_state;
    }

    fn visit_array_mut(&mut self, node: &mut Array) {
        // Format any arrays within dependencies to be on the same line.
        if matches!(
            self.state,
            VisitState::Dependencies | VisitState::SubDependencies
        ) {
            node.fmt();
        }
    }
}
