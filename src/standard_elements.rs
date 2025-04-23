use lotus_script::time::delta;

pub fn exponential_approach(old_value: f32, exponent: f32, target: f32) -> f32 {
    let factor = 1.0 - (-delta() * exponent).exp();
    old_value + factor * (target - old_value)
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

    pub fn on_change(&self, f: impl Fn(&T) + 'static, _id: String)
    where
        T: 'static,
    {
        let mut r = self.receiver.clone();

        lotus_rt::spawn(async move {
            while (r.changed().await).is_ok() {
                let v = r.borrow_and_update();
                // log::debug!("on_change, id = {}", _id);
                f(&v);
            }
        });
    }

    pub async fn await_change(&mut self) {
        let _ = self.receiver.changed().await.is_ok();
    }
}

impl<T: 'static> OnChange for Shared<T> {
    fn on_change(&self, f: Box<dyn Fn()>) {
        self.on_change(move |_| f(), "multiple_on_change".to_string());
    }
}

impl<T: 'static> OnChange for Option<Shared<T>> {
    fn on_change(&self, f: Box<dyn Fn()>) {
        if let Some(sh) = self {
            sh.on_change(move |_| f(), "multiple_on_change".to_string());
        }
    }
}

pub fn multiple_on_change(shareds: &[&dyn OnChange], f: impl Fn() + 'static + Clone) {
    for sh in shareds {
        sh.on_change(Box::new(f.clone()));
    }
}

pub trait OnChange {
    fn on_change(&self, f: Box<dyn Fn()>);
}

impl<T: PartialEq + Clone + std::fmt::Debug> Shared<T> {
    pub fn set_only_on_change(&self, value: T) {
        if *self.receiver.borrow() != value {
            self.sender.send(value).ok();
        }
    }
}

impl Shared<f32> {
    pub fn switch(&self, b: bool) -> f32 {
        if b {
            self.get()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_on_change_callback() {
        // Erstelle einen atomaren Zähler, der zwischen Threads geteilt werden kann
        let counter = Arc::new(AtomicUsize::new(0));
        let shared = Shared::new(String::from("test"));

        // Klon des Zählers für die Closure
        let counter_clone = counter.clone();
        shared.on_change(
            move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            },
            "test".to_string(),
        );

        // Warte kurz, damit die on_change Subscription aktiv wird
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Führe mehrere Änderungen durch
        shared.set(String::from("test1"));
        shared.set(String::from("test2"));
        shared.set(String::from("test3"));

        // Warte kurz, damit alle Änderungen verarbeitet werden können
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Überprüfe, ob der Zähler dreimal inkrementiert wurde
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
