mod envelope;
mod osc;
mod types;

use std::any::Any;

pub use envelope::Envelope;
pub use osc::Oscillator;
pub use types::Wave;

pub trait Node<T> {
    fn get_input(&mut self, slot: InputSlot) -> Result<&mut dyn Any, ()>;
    fn set_input(&mut self, input: Box<dyn Any>, slot: InputSlot) -> Result<Box<dyn Any>, ()>;
    fn tick(&mut self) -> T;
    fn press(&mut self);
    fn release(&mut self);
}

pub fn into_input<T>(input: Box<dyn Any>) -> Result<Input<T>, ()>
where
    T: 'static,
{
    match input.downcast::<Input<T>>() {
        Ok(input) => Ok(*input),
        Err(_) => Err(()),
    }
}

pub enum Input<T> {
    Node(Box<dyn Node<T>>),
    Value(T),
}

unsafe impl<T: Sized> Send for Input<T> {}

impl<T: Copy> Input<T> {
    pub fn value(&mut self) -> T {
        match self {
            Input::Node(node) => node.tick(),
            Input::Value(value) => return *value,
        }
    }
}

pub enum InputSlot {
    Input,
    Wave,
    Frequency,
    Amplitude,
}
