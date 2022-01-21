//! Floating point wrapper structs providing a total ordering for comparision and a safe
//! hash implementation, allowing it to be used as the key in a HashMap / HashSet etc.
//!
//! ## Example Usage
//!
//! ```rust
//! use std::collections::HashMap;
//! use total_float_wrap::TotalF64;
//!
//! let mut map: HashMap<TotalF64, u64> = HashMap::new();
//! 
//! map.insert(1.0.into(), 10);
//!
//! assert_eq!(map.get(&1.0.into()), Some(&10));
//! ```

mod total_f32;
pub use total_f32::TotalF32;

mod total_f64;
pub use total_f64::TotalF64;
