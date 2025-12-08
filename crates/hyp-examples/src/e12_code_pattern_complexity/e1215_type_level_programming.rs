/// E1215: Type-level programming with const evaluation
/// Severity: HIGH
/// LLM confusion: 5 (HIGHEST)
///
/// Description: This code uses Rust's type system to perform computations at compile time,
/// essentially programming with types instead of values. Types are used to encode logic and
/// relationships that are checked and computed by the compiler. This is extremely abstract and
/// confusing because you're not working with runtime data - you're working with compile-time
/// type relationships. It's like meta-programming where types are your variables.
///
/// Mitigation: Avoid type-level programming unless absolutely essential for API safety. When
/// needed, provide extensive documentation. Show concrete examples of what the type-level
/// computation achieves. Consider if runtime checks would be simpler and clearer.
use std::marker::PhantomData;

// PROBLEM E1215: Type-level numbers using Zero-Sized Types
pub struct Zero;
pub struct E1215Succ<N>(PhantomData<N>);

// Type-level addition trait
pub trait Add<Rhs> {
    type Output;
}

// PROBLEM E1215: Type-level arithmetic implemented through traits
impl<N> Add<Zero> for N {
    type Output = N;
}

impl<N, M> Add<E1215Succ<M>> for N
where
    N: Add<M>,
{
    type Output = E1215Succ<N::Output>;
}

// PROBLEM E1215: Type-level comparison
pub trait Greater<Rhs> {
    type Output;
}

// PROBLEM E1215: Using PhantomData to carry type information
pub struct Vec<T, N> {
    data: std::vec::Vec<T>,
    _size: PhantomData<N>,
}

impl<T, N> Vec<T, N> {
    pub fn e1215_type_level_programming() -> Self
    where
        N: Default,
    {
        Vec {
            data: std::vec::Vec::new(),
            _size: PhantomData,
        }
    }
}

// PROBLEM E1215: Type-level state machine
pub struct Locked;
pub struct Unlocked;

pub struct StateMachine<State> {
    data: String,
    _state: PhantomData<State>,
}

impl StateMachine<Unlocked> {
    pub fn new(data: String) -> Self {
        StateMachine {
            data,
            _state: PhantomData,
        }
    }

    pub fn lock(self) -> StateMachine<Locked> {
        StateMachine {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl StateMachine<Locked> {
    // PROBLEM E1215: Method only exists for specific type-level state
    pub fn process(&mut self) {
        self.data.push_str("_processed");
    }

    pub fn unlock(self) -> StateMachine<Unlocked> {
        StateMachine {
            data: self.data,
            _state: PhantomData,
        }
    }
}

// PROBLEM E1215: Type-level list
pub struct Nil;
pub struct Cons<Head, Tail>(PhantomData<(Head, Tail)>);

pub trait Length {
    const VALUE: usize;
}

impl Length for Nil {
    const VALUE: usize = 0;
}

impl<H, T: Length> Length for Cons<H, T> {
    const VALUE: usize = 1 + T::VALUE;
}

pub fn e1215_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = std::marker::PhantomData::<E1215Succ<E1215Succ<Zero>>>;
    Ok(())
}
