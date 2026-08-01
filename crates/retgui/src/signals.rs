use std::cell::RefCell;
use std::rc::Rc;

type Subscriber = Rc<dyn Fn()>;

struct SignalInner<T> {
    value: T,
    subscribers: Vec<Subscriber>,
}

#[derive(Clone)]
pub struct Signal<T>(Rc<RefCell<SignalInner<T>>>);

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(SignalInner {
            value,
            subscribers: Vec::new(),
        })))
    }

    pub fn get(&self) -> T {
        self.0.borrow().value.clone()
    }

    pub fn set(&self, new_value: T) {
        self.0.borrow_mut().value = new_value;
        let subscribers = self.0.borrow().subscribers.clone();
        for subscriber in subscribers {
            subscriber();
        }
    }

    pub fn subscribe(&self, subscriber: Subscriber) {
        self.0.borrow_mut().subscribers.push(subscriber);
    }
}

/*pub fn create_for<T: Clone + 'static, K: std::hash::Hash + Eq + Copy + 'static>(
    container: Container,
    list_signal: Signal<Vec<T>>,
    key_fn: impl Fn(&T) -> K + 'static,
    view_fn: impl Fn(&T) -> Container + 'static,
) {
    let element_cache = Rc::new(RefCell::new(HashMap::<K, Container>::new()));
    let c = container.clone();

    let value = list_signal.clone();
    let runner = Rc::new(move || {
        let current_data = value.get();
        let mut cache = element_cache.borrow_mut();

        let mut new_children = Vec::new();
        for item in current_data.iter() {
            let key = key_fn(item);
            let child = cache.entry(key).or_insert_with(|| view_fn(item));
            new_children.push(child.clone());
        }

        c.remove_all_children();
        for child in new_children {
            c.clone().push(child);
        }

        let current_keys: Vec<K> = current_data.iter().map(&key_fn).collect();
        cache.retain(|k, _| current_keys.contains(k));
    });

    runner();
    list_signal.subscribe(runner);
}*/

impl<T: Clone + 'static> Signal<T> {
    pub fn map<U: Clone + 'static, F: Fn(T) -> U + 'static>(&self, f: F) -> Signal<U> {
        let derived = Signal::new(f(self.get()));
        let d = derived.clone();
        let signal = self.clone();

        self.subscribe(Rc::new(move || {
            d.set(f(signal.get()));
        }));

        derived
    }
}

pub trait Bindable<T>: 'static {
    fn bind(self, f: impl Fn(T) + 'static);
}

impl<T: 'static> Bindable<T> for T {
    #[inline]
    fn bind(self, f: impl Fn(T) + 'static) {
        f(self);
    }
}

impl<T: Clone + 'static> Bindable<T> for Signal<T> {
    fn bind(self, f: impl Fn(T) + 'static) {
        let signal = self.clone();

        let runner = Rc::new(move || {
            f(signal.get());
        });

        runner();
        self.subscribe(runner);
    }
}
