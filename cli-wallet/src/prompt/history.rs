use std::collections::VecDeque;

use dialoguer::History;

pub struct PromptHistory {
    max: usize,
    history: VecDeque<String>,
}

impl Default for PromptHistory {
    fn default() -> Self {
        PromptHistory {
            max: 25,
            history: VecDeque::new(),
        }
    }
}

impl<T: ToString> History<T> for PromptHistory {
    fn read(&self, pos: usize) -> Option<String> {
        self.history.get(pos).cloned()
    }

    fn write(&mut self, val: &T) {
        let entry = val.to_string();
        if self.history.contains(&entry) {
            return;
        }
        if self.history.len() == self.max {
            self.history.pop_back();
        }
        self.history.push_front(entry);
    }
}
