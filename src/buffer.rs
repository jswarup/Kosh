//-- buffer.rs --------------------------------------------------------------------------------------------------------------------
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use anyhow::{anyhow, Result};

pub struct Buffer<ValueT>
{
    pub _Data: Vec<ValueT>,
}

impl<ValueT> Buffer<ValueT>
{
    /// Create a new Buffer with the provided elements.
    pub fn New(items: Vec<ValueT>) -> Self
    {
        Self
        {
            _Data: items,
        }
    }

    /// Create a new Buffer pre-allocated with the given capacity.
    pub fn WithCapacity(capacity: usize) -> Self
    {
        Self
        {
            _Data: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the buffer.
    pub fn Len(&self) -> usize
    {
        self._Data.len()
    }

    /// Returns true if the buffer contains no elements.
    pub fn IsEmpty(&self) -> bool
    {
        self._Data.is_empty()
    }

    /// Returns a reference to the element at the given index, or None if out of bounds.
    pub fn Get(&self, index: usize) -> Option<&ValueT>
    {
        self._Data.get(index)
    }

    /// Returns a mutable reference to the element at the given index, or None if out of bounds.
    pub fn GetMut(&mut self, index: usize) -> Option<&mut ValueT>
    {
        self._Data.get_mut(index)
    }

    /// Sets the element at the given index to a new value.
    /// Returns Ok(()) if successful, or an Error if index is out of bounds.
    pub fn Set(&mut self, index: usize, value: ValueT) -> Result<()>
    {
        if index < self._Data.len()
        {
            self._Data[index] = value;
            Ok(())
        }
        else
        {
            Err(anyhow!(
                "Index {} is out of bounds for buffer of length {}",
                index,
                self._Data.len()
            ))
        }
    }

    /// Push an element to the end of the buffer.
    pub fn Push(&mut self, value: ValueT)
    {
        self._Data.push(value);
    }
}

// Implement standard Index and IndexMut traits for array-like indexing
impl<ValueT> std::ops::Index<usize> for Buffer<ValueT>
{
    type Output = ValueT;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self._Data[index]
    }
}

impl<ValueT> std::ops::IndexMut<usize> for Buffer<ValueT>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output
    {
        &mut self._Data[index]
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn TestBufferBasicOperations()
    {
        let mut buffer = Buffer::New(vec![10, 20, 30]);
        assert_eq!(buffer.Len(), 3);
        assert!(!buffer.IsEmpty());

        assert_eq!(buffer.Get(1), Some(&20));
        assert_eq!(buffer.Get(5), None);

        // Test mutating elements
        *buffer.GetMut(1).unwrap() = 25;
        assert_eq!(buffer.Get(1), Some(&25));

        // Test Set method
        assert!(buffer.Set(2, 35).is_ok());
        assert_eq!(buffer.Get(2), Some(&35));
        assert!(buffer.Set(5, 50).is_err());

        // Test push
        buffer.Push(40);
        assert_eq!(buffer.Len(), 4);
        assert_eq!(buffer.Get(3), Some(&40));

        // Test indexing trait
        assert_eq!(buffer[0], 10);
        buffer[0] = 15;
        assert_eq!(buffer[0], 15);
    }

    #[test]
    fn TestBufferWithCapacity()
    {
        let mut buffer: Buffer<i32> = Buffer::WithCapacity(10);
        assert_eq!(buffer.Len(), 0);
        assert!(buffer.IsEmpty());
        buffer.Push(100);
        assert_eq!(buffer.Len(), 1);
        assert_eq!(buffer.Get(0), Some(&100));
    }
}
