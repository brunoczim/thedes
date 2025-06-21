use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering::*},
};

pub fn open(goal: usize) -> (Logger, Monitor) {
    let shared = Arc::new(Shared {
        goal,
        current: AtomicUsize::new(0),
        status: std::sync::Mutex::new(Vec::new()),
    });

    let logger =
        Logger { shared: shared.clone(), status_index: shared.enter() };
    let monitor = Monitor { shared };

    (logger, monitor)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    current: usize,
    status: String,
}

impl Progress {
    pub fn current(&self) -> usize {
        self.current
    }

    pub fn status(&self) -> &str {
        &self.status
    }
}

#[derive(Debug)]
struct Shared {
    goal: usize,
    current: AtomicUsize,
    status: std::sync::Mutex<Vec<Option<Box<str>>>>,
}

impl Shared {
    pub fn goal(&self) -> usize {
        self.goal
    }

    pub fn read_status(&self) -> String {
        let statuses = self.status.lock().expect("poisoned lock");
        let mut status = String::new();

        for maybe in &*statuses {
            if let Some(item) = maybe {
                if !status.is_empty() {
                    status += " > ";
                }
                status += item;
            }
        }

        status
    }

    pub fn read_current(&self) -> usize {
        self.current.load(Relaxed)
    }

    pub fn read(&self) -> Progress {
        Progress { status: self.read_status(), current: self.read_current() }
    }

    pub fn increment(&self) {
        self.current.fetch_add(1, Relaxed);
    }

    pub fn set_status(&self, index: usize, status: &str) {
        self.status.lock().expect("poisoned lock")[index] = Some(status.into());
    }

    pub fn enter(&self) -> usize {
        let mut statuses = self.status.lock().expect("poisoned lock");
        let index = statuses.len();
        statuses.push(Some(Box::default()));
        index
    }

    pub fn leave(&self, index: usize) {
        let mut statuses = self.status.lock().expect("poisoned lock");
        statuses[index] = None;

        while statuses.last().is_some_and(Option::is_none) {
            statuses.pop();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Monitor {
    shared: Arc<Shared>,
}

impl Monitor {
    pub fn goal(&self) -> usize {
        self.shared.goal()
    }

    pub fn read(&self) -> Progress {
        self.shared.read()
    }
}

#[derive(Debug)]
pub struct Logger {
    shared: Arc<Shared>,
    status_index: usize,
}

impl Logger {
    pub fn goal(&self) -> usize {
        self.shared.goal()
    }

    pub fn read(&self) -> Progress {
        self.shared.read()
    }

    pub fn increment(&self) {
        self.shared.increment();
    }

    pub fn set_status(&self, status: &str) {
        self.shared.set_status(self.status_index, status);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.shared.leave(self.status_index);
    }
}
