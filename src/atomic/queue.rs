use crate::{
    Error
};

use {
    std::sync::{ Arc, RwLock },
    std::cmp::min,
    std::time::Duration,

    tokio::sync::Notify,
};

static REASONABLE: usize = 40;

#[derive(Debug)]
struct QueueState<T> {
    items: Vec<T>,
    latest: usize,
}

#[derive(Debug, Clone)]
pub struct Queue<T> {
    inner: Arc<RwLock<QueueState<T>>>,
    new_notif: Arc<Notify>,
}

impl<T: Clone> Queue<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(QueueState {
                items: Vec::new(),
                latest: 0,
            })),
            new_notif: Arc::new(Notify::new())
        }
    }

    pub fn push(&self, item: T) -> Result<(), Error> {
        let mut state = self.inner.write().map_err(|_| "Lock poisoned")?;
        
        state.items.push(item);
        state.latest += 1;

        self.new_notif.notify_waiters();

        Ok(())
    }

    pub fn get_all_since(&self, latest: &mut usize) -> Vec<T> {
        let state = self.inner.read().unwrap();

        if *latest >= state.items.len() {
            return Vec::new();
        }

        *latest = state.items.len();
        state.items[*latest..].to_vec()
    }

    pub fn get_last(&self, amount: i32) -> Vec<T> {
        let mut msg_idx = (self.len() as i32 - amount) as usize;
        self.get_all_since(&mut msg_idx)
    }

    pub async fn on_new(&self, latest: &mut usize) -> Vec<T> {
        loop {
            {
                let state = self.inner.read().unwrap();
                if state.latest > *latest {
                    let items = state.items[*latest..].to_vec();

                    *latest = state.items.len(); 
                    return items;
                }
            }

            self.new_notif.notified().await;
        }
    }

    pub fn get_reasonable(&self, latest: &mut usize) -> Vec<T> {
        let state = self.inner.read().unwrap();

        *latest = min(*latest, state.items.len().saturating_sub(REASONABLE));

        let items = state.items[*latest..].to_vec();
        *latest = state.items.len();
        items
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().items.len()
    }
}

pub fn now_time() -> Duration {
    use std::time::{ SystemTime, UNIX_EPOCH };

    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}
