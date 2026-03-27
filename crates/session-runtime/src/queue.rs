#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueItem {
    Steering(String),
    FollowUp(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionQueue {
    items: Vec<QueueItem>,
}

impl SessionQueue {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_items(items: Vec<QueueItem>) -> Self {
        Self { items }
    }

    pub fn push_steering(&mut self, message: impl Into<String>) {
        self.items.push(QueueItem::Steering(message.into()));
    }

    pub fn push_follow_up(&mut self, message: impl Into<String>) {
        self.items.push(QueueItem::FollowUp(message.into()));
    }

    pub fn items(&self) -> &[QueueItem] {
        &self.items
    }

    pub fn steering_items(&self) -> impl Iterator<Item = &str> {
        self.items.iter().filter_map(|item| match item {
            QueueItem::Steering(message) => Some(message.as_str()),
            QueueItem::FollowUp(_) => None,
        })
    }

    pub fn follow_up_items(&self) -> impl Iterator<Item = &str> {
        self.items.iter().filter_map(|item| match item {
            QueueItem::Steering(_) => None,
            QueueItem::FollowUp(message) => Some(message.as_str()),
        })
    }

    pub fn drain_follow_up_items(&mut self) -> Vec<String> {
        let mut drained = Vec::new();
        self.items.retain(|item| match item {
            QueueItem::Steering(_) => true,
            QueueItem::FollowUp(message) => {
                drained.push(message.clone());
                false
            }
        });
        drained
    }
}
