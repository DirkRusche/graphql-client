//! Trimming of a query document to the parts needed by a single operation:
//! the operation itself and the fragments it transitively references.

use crate::normalization::Normalization;
use graphql_parser::query::{
    Definition, Document, FragmentDefinition, OperationDefinition, Selection, SelectionSet,
};
use std::collections::{BTreeMap, BTreeSet};

type StaticDocument = Document<'static, String>;

/// Build the minimal query string for the given operation: its definition plus
/// the fragment definitions it transitively spreads, in document order.
///
/// Returns `None` when the operation cannot be identified unambiguously by
/// name, or when a spread references a fragment missing from the document. The
/// caller should then fall back to the full document text.
pub(crate) fn trim_query_for_operation(
    document: &StaticDocument,
    operation_name: &str,
    normalization: Normalization,
) -> Option<String> {
    let target_name = normalization.operation(operation_name);

    let mut selected_operation: Option<&OperationDefinition<'static, String>> = None;
    let mut fragments: BTreeMap<&str, &FragmentDefinition<'static, String>> = BTreeMap::new();

    for definition in &document.definitions {
        match definition {
            Definition::Operation(op) => {
                let name = operation_definition_name(op)?;
                if normalization.operation(name) == target_name {
                    if selected_operation.is_some() {
                        return None;
                    }
                    selected_operation = Some(op);
                }
            }
            Definition::Fragment(fragment) => {
                fragments.insert(fragment.name.as_str(), fragment);
            }
        }
    }

    let selected_operation = selected_operation?;

    let mut used_fragments: BTreeSet<&str> = BTreeSet::new();
    let mut pending: Vec<&str> = Vec::new();
    collect_fragment_spreads(operation_selection_set(selected_operation), &mut pending);

    while let Some(name) = pending.pop() {
        if !used_fragments.insert(name) {
            continue;
        }
        let fragment = fragments.get(name)?;
        collect_fragment_spreads(&fragment.selection_set, &mut pending);
    }

    let definitions = document
        .definitions
        .iter()
        .filter(|definition| match definition {
            Definition::Operation(op) => std::ptr::eq(op, selected_operation),
            Definition::Fragment(fragment) => used_fragments.contains(fragment.name.as_str()),
        })
        .cloned()
        .collect();

    Some(Document { definitions }.to_string())
}

fn operation_definition_name<'a>(
    operation: &'a OperationDefinition<'static, String>,
) -> Option<&'a str> {
    match operation {
        OperationDefinition::Query(q) => q.name.as_deref(),
        OperationDefinition::Mutation(m) => m.name.as_deref(),
        OperationDefinition::Subscription(s) => s.name.as_deref(),
        OperationDefinition::SelectionSet(_) => None,
    }
}

fn operation_selection_set<'a>(
    operation: &'a OperationDefinition<'static, String>,
) -> &'a SelectionSet<'static, String> {
    match operation {
        OperationDefinition::Query(q) => &q.selection_set,
        OperationDefinition::Mutation(m) => &m.selection_set,
        OperationDefinition::Subscription(s) => &s.selection_set,
        OperationDefinition::SelectionSet(selection_set) => selection_set,
    }
}

fn collect_fragment_spreads<'a>(
    selection_set: &'a SelectionSet<'static, String>,
    spreads: &mut Vec<&'a str>,
) {
    for selection in &selection_set.items {
        match selection {
            Selection::Field(field) => collect_fragment_spreads(&field.selection_set, spreads),
            Selection::InlineFragment(inline) => {
                collect_fragment_spreads(&inline.selection_set, spreads)
            }
            Selection::FragmentSpread(spread) => spreads.push(spread.fragment_name.as_str()),
        }
    }
}
