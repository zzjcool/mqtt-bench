#[derive(Debug, Clone)]
pub(crate) struct Subscription {
    pub(crate) topic_filter: String,
    pub(crate) qos: i32,
}

impl Subscription {
    pub(crate) fn new(topic_filter: String, qos: i32) -> Subscription {
        Self { topic_filter, qos }
    }
}
