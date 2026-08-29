use std::sync::{Arc, Mutex};

pub struct EventHandle<T>
where
    T: Clone + Send + Sync + 'static,
{
    id: usize,
    event: Arc<Mutex<EventCore<T>>>,
}

struct EventListener<T>
where
    T: Clone + Send + Sync + 'static,
{
    id: usize,
    callback: Box<dyn Fn(T) + Send + Sync>,
}

struct EventCore<T>
where
    T: Clone + Send + Sync + 'static,
{
    listeners: Vec<EventListener<T>>,
    listener_id_counter: usize,
}

pub struct Event<T>
where
    T: Clone + Send + Sync + 'static,
{
    core: Arc<Mutex<EventCore<T>>>,
}

impl<T> Event<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Event {
            core: Arc::new(Mutex::new(EventCore {
                listeners: Vec::new(),
                listener_id_counter: 0,
            })),
        }
    }

    pub fn emit(&self, data: T) {
        // Emit the event to all listeners
        let core = self.core.lock().unwrap();
        for listener in &core.listeners {
            (listener.callback)(data.clone());
        }
    }

    pub fn listen<F>(&self, _callback: F) -> EventHandle<T>
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        // Subscribe to the event with the provided callback
        let mut core = self.core.lock().unwrap();
        let listener_id = core.listener_id_counter;
        core.listener_id_counter += 1;
        core.listeners.push(EventListener {
            id: listener_id,
            callback: Box::new(_callback),
        });

        EventHandle {
            id: listener_id,
            event: Arc::clone(&self.core),
        }
    }
}

impl<T> Drop for EventHandle<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        // Unsubscribe from the event when the handle is dropped
        let mut core = self.event.lock().unwrap();
        core.listeners.retain(|listener| listener.id != self.id);
    }
}

impl<T> Default for Event<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Event<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Event {
            core: Arc::clone(&self.core),
        }
    }
}

impl<T> std::fmt::Debug for Event<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("listeners", &self.core.lock().unwrap().listeners.len())
            .finish()
    }
}
