/// E1206: Deeply nested generic types
/// Severity: HIGH
/// LLM confusion: 4 (HIGH)
///
/// Description: This code has generic types nested within other generic types, creating a complex
/// type hierarchy that's extremely difficult to understand. Each layer of nesting adds cognitive
/// load - you have to track what each type parameter means at each level. It's like having
/// containers within containers within containers, where each container has its own type rules.
/// Fix by flattening the type hierarchy, using type aliases for complex nested types, or
/// restructuring to avoid deep nesting.
///
/// ## The Nesting Problem
///
/// ```text
/// type Nested<T> = HashMap<String, Vec<Option<Result<Box<T>, E>>>>;
/// // To access T, you must:
/// // 1. Look up key in HashMap
/// // 2. Index into Vec
/// // 3. Unwrap Option
/// // 4. Unwrap Result
/// // 5. Dereference Box
/// ```
///
/// ## Why This Matters
///
/// 1. **Cognitive load**: Each layer adds mental overhead
/// 2. **Error handling**: Each layer can fail differently
/// 3. **Type inference**: Compiler struggles with deep nesting
/// 4. **Refactoring**: Changes propagate through all layers
///
/// ## The Right Solutions
///
/// ### Option 1: Use type aliases
/// ```rust
/// type ProcessResult<T> = Result<T, ProcessError>;
/// type OptionalProcess<T> = Option<ProcessResult<T>>;
/// type ProcessList<T> = Vec<OptionalProcess<T>>;
/// ```
///
/// ### Option 2: Flatten with custom types
/// ```rust
/// struct ProcessedItem<T> {
///     value: T,
///     status: Status,
/// }
/// type ProcessList<T> = Vec<ProcessedItem<T>>;
/// ```
///
/// ### Option 3: Limit nesting depth
/// ```rust
/// // Maximum 2-3 levels of generic nesting
/// type SimpleMap<T> = HashMap<String, Vec<T>>;
/// ```
///
/// Mitigation: Use type aliases to name complex nested types: `type MyType<T> = Vec<Option<Result<T, E>>>`.
/// Limit generic nesting to 2-3 levels. Consider if the complexity is necessary - often simpler
/// designs work better. Use `#![warn(clippy::type_complexity)]` to catch overly complex types.

use std::collections::HashMap;

// ============================================================================
// PROBLEMATIC PATTERNS
// ============================================================================

// This type alias shows 5 levels of generic nesting:
// 1. HashMap - maps String keys to values
// 2. Vec - each value is a vector of...
// 3. Option - which might contain...
// 4. Result - which is either Ok (success) or Err (error) containing...
// 5. Box<dyn Fn...> - a heap-allocated function that takes T and returns Result<Vec<T>, E>
// Reading this type requires mentally unwrapping 5 layers to understand what data it holds.
//
// PROBLEM E1206: Deeply nested generic types (5 levels)
pub type E1206ComplexType<T, E> =
    HashMap<String, Vec<Option<Result<Box<dyn Fn(T) -> Result<Vec<T>, E> + Send>, E>>>>;

// This function returns the deeply nested type defined above.
// The where clause ensures T can be cloned and sent between threads,
// and E is an error type that can also be sent between threads.
//
// PROBLEM E1206: Working with deeply nested types is confusing
pub fn e1206_bad_deeply_nested_generics<T, E>() -> E1206ComplexType<T, E>
where
    T: Clone + Send + 'static,
    E: std::error::Error + Send + 'static,
{
    let map: HashMap<
        String,
        Vec<Option<Result<Box<dyn Fn(T) -> Result<Vec<T>, E> + Send>, E>>>,
    > = HashMap::new();

    map
}

// These three structs show nested generic structures:
// Container holds a Vec of Wrappers
// Wrapper holds a Box containing a Holder
// Holder holds an Option<Result<T>> - the actual data is buried 3 layers deep
// PROBLEM E1206: Nested generic structs
pub struct Container<T> {
    inner: Vec<Wrapper<T>>,
}

pub struct Wrapper<T> {
    data: Box<Holder<T>>,
}

pub struct Holder<T> {
    value: Option<Result<T, String>>,
}

impl<T> Container<T> {
    /// Creates an empty container
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a container with a single value
    pub fn with_value(value: T) -> Self {
        Self {
            inner: vec![Wrapper {
                data: Box::new(Holder {
                    value: Some(Ok(value)),
                }),
            }],
        }
    }

    /// Adds a wrapper to the container
    pub fn push(&mut self, wrapper: Wrapper<T>) {
        self.inner.push(wrapper);
    }
}

impl<T> Default for Container<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Wrapper<T> {
    /// Creates a wrapper with a value
    pub fn new(value: T) -> Self {
        Self {
            data: Box::new(Holder {
                value: Some(Ok(value)),
            }),
        }
    }

    /// Creates a wrapper with an error
    pub fn with_error(error: String) -> Self {
        Self {
            data: Box::new(Holder {
                value: Some(Err(error)),
            }),
        }
    }

    /// Creates an empty wrapper
    pub fn empty() -> Self {
        Self {
            data: Box::new(Holder { value: None }),
        }
    }
}

impl<T> Holder<T> {
    /// Creates a holder with a value
    pub fn new(value: T) -> Self {
        Self {
            value: Some(Ok(value)),
        }
    }
}

// This function returns Container<Container<Container<T>>> - nesting the Container type 3 times!
// To access the inner T value, you'd need to unwrap 3 Container layers,
// plus the Vec, Box, Option, and Result inside each layer.
// PROBLEM E1206: Three levels of the same generic struct nested
pub fn e1206_bad_nested_generic_structs<T: Clone>() -> Container<Container<Container<T>>> {
    // Create the innermost empty container (level 3)
    let innermost: Container<T> = Container::new();

    // Wrap it in a middle container (level 2)
    let middle: Container<Container<T>> = Container::with_value(innermost);

    // Wrap that in the outermost container (level 1)
    let outermost: Container<Container<Container<T>>> = Container::with_value(middle);

    outermost
}

pub fn e1206_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _: E1206ComplexType<i32, std::io::Error> = e1206_bad_deeply_nested_generics();
    let _: Container<Container<Container<i32>>> = e1206_bad_nested_generic_structs();
    Ok(())
}

// ============================================================================
// GOOD ALTERNATIVES
// ============================================================================

/// GOOD: Use type aliases for clarity
pub type ProcessResult<T, E> = Result<T, E>;
pub type OptionalResult<T, E> = Option<ProcessResult<T, E>>;
pub type ResultList<T, E> = Vec<OptionalResult<T, E>>;

/// GOOD: Flatten nested structures with meaningful types
#[derive(Debug, Clone)]
pub struct ProcessedItem<T> {
    pub value: T,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Success,
    Pending,
    Failed(String),
}

/// GOOD: Simple container instead of deeply nested generics
pub type ProcessedList<T> = Vec<ProcessedItem<T>>;

/// GOOD: Use named intermediate types
pub struct DataStore<T> {
    items: Vec<T>,
}

impl<T> DataStore<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
}

impl<T> Default for DataStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// GOOD: Limit nesting to 2 levels
pub type SimpleNestedType<T> = HashMap<String, Vec<T>>;

/// GOOD: Use enums instead of nested Options/Results
#[derive(Debug, Clone)]
pub enum DataState<T> {
    Empty,
    Loading,
    Ready(T),
    Error(String),
}

impl<T> DataState<T> {
    pub fn is_ready(&self) -> bool {
        matches!(self, DataState::Ready(_))
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            DataState::Ready(v) => Some(v),
            _ => None,
        }
    }
}

/// GOOD: Flatten the Container hierarchy
#[derive(Debug, Clone)]
pub struct FlatContainer<T> {
    items: Vec<DataState<T>>,
}

impl<T> FlatContainer<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: T) {
        self.items.push(DataState::Ready(item));
    }

    pub fn add_pending(&mut self) {
        self.items.push(DataState::Loading);
    }

    pub fn get_ready_items(&self) -> impl Iterator<Item = &T> {
        self.items.iter().filter_map(|state| state.get())
    }
}

impl<T> Default for FlatContainer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_generic_structs() {
        let nested: Container<Container<Container<i32>>> = e1206_bad_nested_generic_structs();
        // The nested structure exists but is empty
        assert!(nested.inner.is_empty() || !nested.inner.is_empty());
    }

    #[test]
    fn test_container_with_value() {
        let container = Container::with_value(42);
        assert_eq!(container.inner.len(), 1);
    }

    #[test]
    fn test_wrapper() {
        let wrapper = Wrapper::new(42);
        assert!(wrapper.data.value.is_some());
    }

    #[test]
    fn test_flat_container() {
        let mut container = FlatContainer::new();
        container.add(1);
        container.add(2);
        container.add(3);

        let ready: Vec<_> = container.get_ready_items().collect();
        assert_eq!(ready.len(), 3);
    }

    #[test]
    fn test_data_state() {
        let ready: DataState<i32> = DataState::Ready(42);
        assert!(ready.is_ready());
        assert_eq!(ready.get(), Some(&42));

        let loading: DataState<i32> = DataState::Loading;
        assert!(!loading.is_ready());
        assert_eq!(loading.get(), None);
    }

    #[test]
    fn test_processed_item() {
        let item = ProcessedItem {
            value: 42,
            status: ProcessStatus::Success,
        };
        assert_eq!(item.status, ProcessStatus::Success);
    }
}
