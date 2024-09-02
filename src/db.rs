// use bytes::Bytes;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::time::{self, Duration, Instant};

pub(crate) struct DbDropGuard {
    db: Db,
}

impl DbDropGuard {
    pub(crate) fn new() -> DbDropGuard {
        DbDropGuard { db: Db::new() }
    }

    pub(crate) fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Drop for DbDropGuard {
    fn drop(&mut self) {
        self.db.shutdown_purge_task();
    }
}

#[derive(Clone)]
pub(crate) struct Db {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
    background_task: Notify,
}

struct State {
    entries: HashMap<String, Entry>,
    expirations: BTreeSet<(Instant, String)>,
    shutdown: bool,
}

struct Entry {
    data: String, // In the future, could use Bytes instead
    expires_at: Option<Instant>,
}

impl Db {
    pub(crate) fn new() -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                expirations: BTreeSet::new(),
                shutdown: false,
            }),
            background_task: Notify::new(),
        });

        // Start background task for purging expired entries
        tokio::spawn(purge_expired_tasks(shared.clone()));

        Db { shared }
    }

    pub(crate) fn get(&self, key: &str) -> Option<String> {
        let state = self.shared.state.lock().unwrap();
        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub(crate) fn list(&self, limit: Option<usize>) -> Vec<String> {
        let state = self.shared.state.lock().unwrap();
        let mut vec: Vec<String> = Vec::new();
        let limit = limit.unwrap_or(10);
        for (key, entry) in state.entries.iter().take(limit) {
            vec.push(format!("{}: {}", key, entry.data));
        }
        vec
    }

    pub(crate) fn set(&self, key: String, value: String, expiry: Option<Duration>) {
        let mut state = self.shared.state.lock().unwrap();

        let mut notify = false;

        let expires_at = expiry.map(|duration| {
            let when = Instant::now() + duration;
            notify = state
                .next_expiration()
                .map(|expiration| expiration > when)
                .unwrap_or(true);
            when
        });

        // Returns previous entry value if key overwrites it
        let prev = state.entries.insert(
            key.clone(),
            Entry {
                data: value,
                expires_at,
            },
        );

        // Handle case where value is overwritten and expiry needs to be overwritten
        if let Some(prev) = prev {
            if let Some(when) = prev.expires_at {
                state.expirations.remove(&(when, key.clone()));
            }
        }

        // Handle write to expirations
        if let Some(when) = expires_at {
            state.expirations.insert((when, key));
        }

        // Drop state so background task can operate
        drop(state);

        // Notify background task to update its state to reflect a new expiration entry
        if notify {
            self.shared.background_task.notify_one();
        }
    }

    fn shutdown_purge_task(&self) {
        let mut state = self.shared.state.lock().unwrap();
        state.shutdown = true;

        drop(state);
        self.shared.background_task.notify_one();
    }
}

impl Shared {
    // Purge all expired keys and return the `Instant` at which the next key will expire.
    // Background task should sleep until this instant (Timer)
    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut state = self.state.lock().unwrap();

        if state.shutdown {
            return None;
        }

        let state = &mut *state;
        let now = Instant::now();

        while let Some(&(when, ref key)) = state.expirations.iter().next() {
            // This is the **next** expiration
            if when > now {
                return Some(when);
            }

            // Remove entry at key
            state.entries.remove(key);
            // Remove expiry from expirations
            state.expirations.remove(&(when, key.clone()));
        }

        None
    }

    pub fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdown
    }
}

impl State {
    // Return when **next** expiration occurs
    fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .iter()
            .next()
            .map(|expiration| expiration.0)
    }
}

async fn purge_expired_tasks(shared: Arc<Shared>) {
    while !shared.is_shutdown() {
        if let Some(when) = shared.purge_expired_keys() {
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            shared.background_task.notified().await;
        }
    }
}
