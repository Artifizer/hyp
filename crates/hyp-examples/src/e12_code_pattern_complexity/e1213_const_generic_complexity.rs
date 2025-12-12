/// E1213: Const generics with complex constraints
/// Severity: MED
/// LLM confusion: 4 (HIGH)
///
/// Description: Const generics allow using constant values (like array sizes) as generic parameters.
/// When combined with complex trait bounds and type-level computation, this becomes very confusing.
/// It's like having compile-time computation where numbers are part of the type system. Understanding
/// when and why specific const values are required is difficult for LLMs and humans alike.
///
/// Mitigation: Keep const generic parameters simple. Avoid complex const expressions in bounds.
/// Provide clear examples showing what const values are expected. Document why specific const
/// values are constrained. Use type aliases for complex const generic types.

// PROBLEM E1213: Const generic with trait bounds
pub struct E1213FixedBuffer<T, const N: usize>
where
    T: Default + Copy,
{
    data: [T; N],
    len: usize,
}

impl<T, const N: usize> E1213FixedBuffer<T, N>
where
    T: Default + Copy,
{
    pub fn e1213_const_generic_new() -> Self {
        E1213FixedBuffer {
            data: [T::default(); N],
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), &'static str> {
        if self.len >= N {
            return Err("Buffer full");
        }
        self.data[self.len] = value;
        self.len += 1;
        Ok(())
    }
}

// PROBLEM E1213: Const generic arithmetic in trait bounds
pub trait Matrix<const ROWS: usize, const COLS: usize> {
    fn get(&self, row: usize, col: usize) -> f64;
}

// PROBLEM E1213: Multiple const parameters create complex type signatures
pub struct DenseMatrix<const ROWS: usize, const COLS: usize> {
    data: [[f64; COLS]; ROWS],
}

impl<const ROWS: usize, const COLS: usize> Matrix<ROWS, COLS> for DenseMatrix<ROWS, COLS> {
    fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row][col]
    }
}

// PROBLEM E1213: Const generic in complex trait bound
pub fn e1213_bad_process_buffer<T, const N: usize>(buffer: &E1213FixedBuffer<T, N>)
where
    T: Default + Copy + std::fmt::Debug,
{
    // Process the buffer
}

// PROBLEM E1213: Associated const creates indirect complexity
pub trait SizedConfig {
    const BUFFER_SIZE: usize;
}

pub struct SmallConfig;
impl SizedConfig for SmallConfig {
    const BUFFER_SIZE: usize = 64;
}

pub struct Storage<C: SizedConfig> {
    // PROBLEM E1213: Buffer size comes from associated const
    data: Vec<u8>, // Can't use C::BUFFER_SIZE directly in array size without const generics
    _marker: std::marker::PhantomData<C>,
}

pub fn e1213_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _buffer: E1213FixedBuffer<i32, 10> = E1213FixedBuffer::e1213_const_generic_new();
    e1213_bad_process_buffer(&_buffer);
    Ok(())
}
