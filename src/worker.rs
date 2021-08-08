pub struct Workers {
    workers: VecDeque<Worker>,
}

impl Workers {
    pub fn new() -> Workers {
        Workers {
            workers: VecDeque::new(),
        }
    }

    pub fn get_and_reserve_worker(&mut self) -> Option<(usize, String)> {
        for worker in &mut self.workers {
            if worker.reserved == false {
                worker.reserved = true;
                return Some((worker.uid, worker.string_identifier.clone()));
            }
        }
        return None;
    }
}

pub struct Worker {
    uid: usize,
    string_identifier: String,
    reserved: bool,
    //ip_address
    //mac_address
}

impl Worker {
    pub fn new(string_identifier: String) -> Worker {
        Worker {
            uid: WORKER_UID_COUNTER.fetch_add(1, Ordering::SeqCst),
            string_identifier: string_identifier,
            reserved: false,
        }
    }
}