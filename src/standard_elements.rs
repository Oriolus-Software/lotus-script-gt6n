use lotus_script::time::delta;

pub fn exponential_approach(old_value: f32, exponent: f32, target: f32) -> f32 {
    (1.0 - (delta() * -exponent).exp()) * (target - old_value) + old_value
}

#[derive(Debug, Clone)]
pub struct Shared<T> {
    sender: lotus_rt::sync::watch::Sender<T>,
    receiver: lotus_rt::sync::watch::Receiver<T>,
}

impl<T: Clone> Shared<T> {
    pub fn get(&self) -> T {
        self.receiver.borrow().clone()
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Shared<T> {
    pub fn new(init: T) -> Self {
        let (sender, receiver) = lotus_rt::sync::watch::channel(init);
        Self { sender, receiver }
    }

    pub fn set(&self, value: T) {
        self.sender.send(value).ok();
    }
}
