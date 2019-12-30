#[derive(Default)]
pub struct ForgetfulLogQueue<T> {
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
        // pre-increment so that get... works correctly
        self.head += 1;
        self.head = self.head % self.capacity;

        if self.size < self.capacity {
            self.size += 1;
        }

        self.data[self.head] = object;
    }

    pub fn replace_cur(&mut self, object: T) {
        self.data[self.head] = object;
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

    pub fn clear(&mut self) {
        self.data.clear();
        self.size = 0;
    }
}

impl ForgetfulLogQueue<(u64, f64)> {
    pub fn get_or_avg(&self, position: usize) -> (u64, f64) {
        let ret = self.get(position);

        match ret {
            //Some(av) => av.clone(),
            Some(n) => n.clone(),
            None => {
                let sum: f64 = self.data.iter().map(|tp| { tp.1 }).sum();

                (0, sum / self.size() as f64) // put as far in the past as possible to reduce impact
            }
        }
    }
}
