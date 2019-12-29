#[derive(Default)]
pub struct ForgetfulLogQueue<T> {
    //tail: usize,
    head: usize,
    size: usize,
    capacity: usize,
    data: Vec<T>, // vec presized to capacity
}

impl<T> ForgetfulLogQueue<T> {

    pub fn new(capacity: usize) -> ForgetfulLogQueue<T> {
        ForgetfulLogQueue {
            head: 0,
            size: 0,
            capacity,
            data: Vec::with_capacity(capacity),
        }
    }

    /// Get an item [position] places into the past
    ///
    /// Returns None if provided position is greater than capacity or the current size of the queue
    pub fn get(&self, position: usize) -> Option<&T> {
        if position > self.size {
            None
        } else {
            let index = (self.head as i32) - (position as i32);
            if index < 0 {
                Some(&self.data[(index + self.capacity as i32) as usize])
            } else {
                Some(&self.data[index as usize])
            }
        }
    }

    pub fn push(&mut self, object: T) {
        if self.size < self.capacity {
            self.size += 1;
        }

        self.data[self.head] = object;
        self.head += 1;
        self.head = self.head % self.capacity;
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }

    // TODO: consider moving to returning an Iter, may need to constrain type (no dyn)
    pub fn all(&self) -> &Vec<T> {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
