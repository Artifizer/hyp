/// E1212: Generic Associated Types (GATs) complexity
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: Generic Associated Types (GATs) allow associated types to have their own generic
/// parameters, creating a very complex layer of abstraction. This means a trait can have an
/// associated type that itself is generic over some lifetime or type. This is extremely confusing
/// because you have generics within generics within traits. It's like having a template that
/// returns another template. Fix by simplifying the trait design or avoiding GATs when possible.
///
/// Mitigation: Avoid GATs unless absolutely necessary for your API. When needed, provide extensive
/// documentation and examples. Use concrete types in examples to show how GATs are meant to be used.
/// Consider if a simpler design without GATs would work.

// This trait is like an iterator, but with a twist - the Item type has its own generic parameter!
// Item<'a> means the type of Item depends on the lifetime 'a passed to it.
// This is a GAT (Generic Associated Type). Regular associated types don't have their own parameters.
// The where Self: 'a means the iterator must live at least as long as the items it produces.
//
// PROBLEM E1212: Generic Associated Type - associated type has its own generic parameter
pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

// WindowsMut produces sliding windows over a mutable slice.
// It holds a reference to mutable data ('data) and returns windows of that data.
//
// PROBLEM E1212: Implementing GAT is very confusing
pub struct WindowsMut<'data, T> {
    data: &'data mut [T],
    window_size: usize,
    position: usize,
}

impl<'data, T> LendingIterator for WindowsMut<'data, T> {
    // The Item type is itself generic over lifetime 'a!
    // Item<'a> = &'a mut [T] means "for any lifetime 'a, Item is a mutable slice with that lifetime".
    // This is confusing because the type changes based on what lifetime you pass to it.
    //
    // PROBLEM E1212: The associated type itself is generic over 'a
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn next<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        if self.position + self.window_size > self.data.len() {
            return None;
        }

        let start = self.position;
        let end = start + self.window_size;
        self.position += 1;

        // We use unsafe code here to return a mutable slice.
        // This is a simplified example - real implementations would handle lifetimes more carefully.

        // PROBLEM E1003: Direct use of unsafe code
        unsafe {
            // PROBLEM E1908: No safety documentation
            // PROBLEM E1212: Complex lifetime juggling with GATs
            Some(std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(start),
                end - start,
            ))
        }
    }
}

// This trait has a GAT with TWO parameters - both a lifetime 'a AND a type T!
// Item<'a, T> means the associated type is generic over both a lifetime and a type parameter.
// This creates even more complexity than a single-parameter GAT.
//
// PROBLEM E1212: GAT with multiple type parameters
pub trait StreamingIterator {
    type Item<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn next_item<'a, T>(&'a mut self) -> Option<Self::Item<'a, T>>
    where
        T: Clone;
}

// This function shows using a type with GATs.
// WindowsMut implements LendingIterator (a GAT trait).
// When we call .next(), the Item type's lifetime is inferred from the call context.
//
// PROBLEM E1212: Using GATs is confusing
pub fn e1212_bad_gat_complexity() {
    let mut data = vec![1, 2, 3, 4, 5];
    let mut windows = WindowsMut {
        data: &mut data,
        window_size: 2,
        position: 0,
    };

    while let Some(_window) = windows.next() {
        // Process window
    }
}


pub fn e1212_entry() -> Result<(), Box<dyn std::error::Error>> {
    e1212_bad_gat_complexity();
    Ok(())
}
