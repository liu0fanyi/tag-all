//! Edit Target Type
//!
//! Represents the target being edited in the properties panel.

/// Edit target type - either a Tag, an Item, or multiple Items
#[derive(Clone, Debug)]
pub enum EditTarget {
    /// Tag being edited (id, name)
    Tag(u32, String),
    /// Item being edited (id, text)
    Item(u32, String),
    /// Multiple items being edited (ids)
    MultiItems(Vec<u32>),
}
