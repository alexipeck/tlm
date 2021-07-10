use crate::content::Content;
use std::collections::VecDeque;
pub struct Queue {
    pub priority_queue: VecDeque<Content>,
    pub main_queue: VecDeque<Content>,
}

pub enum QueueType {
    MainQueue,
    PriorityQueue,
    All,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            priority_queue: VecDeque::new(),
            main_queue: VecDeque::new(),
        }
    }

    pub fn print(&mut self) {
        for content in &self.priority_queue {
            println!("{}", content.get_full_path());
        }

        for content in &self.main_queue {
            println!("{}", content.get_full_path());
        }
    }

    pub fn get_index_by_uid_in_queue(
        &mut self,
        uid: usize,
        queue_type: QueueType,
    ) -> Option<usize> {
        match queue_type {
            QueueType::PriorityQueue => {
                for (pos, content) in self.priority_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }
            }
            QueueType::MainQueue => {
                for (pos, content) in self.main_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }
            }
            QueueType::All => {
                for (pos, content) in self.priority_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }

                for (pos, content) in self.main_queue.iter().enumerate() {
                    if content.uid == uid {
                        return Some(pos);
                    }
                }
            }
        }

        return None;
    }

    pub fn get_index_by_uid(&self, uid: usize) -> Option<(usize, QueueType)> {
        for (pos, content) in self.priority_queue.iter().enumerate() {
            if content.uid == uid {
                return Some((pos, QueueType::PriorityQueue));
            }
        }

        for (pos, content) in self.main_queue.iter().enumerate() {
            if content.uid == uid {
                return Some((pos, QueueType::PriorityQueue));
            }
        }

        return None;
    }

    pub fn get_full_queue_length(&mut self) -> usize {
        return self.priority_queue.len() + self.main_queue.len();
    }

    pub fn encode_and_rename_next_unreserved(&mut self, operator: String) {
        let mut working_content: Option<&mut Content> = None;
        for content in &mut self.priority_queue {
            if content.reserved_by == None {
                working_content = Some(content);
            }
        }

        for content in &mut self.main_queue {
            if content.reserved_by == None {
                working_content = Some(content);
            }
        }

        match working_content {
            None => {
                //nothing available for encode
            }
            Some(working_content) => {
                working_content.reserve(operator);
                working_content.encode();
            }
        }
    }

    pub fn get_next_unreserved(&mut self, operator: String) -> Option<usize> {
        for content in &mut self.priority_queue {
            if content.reserved_by == None {
                content.reserve(operator);
                return Some(content.uid);
            }
        }

        for content in &mut self.main_queue {
            if content.reserved_by == None {
                content.reserve(operator);
                return Some(content.uid);
            }
        }

        return None;
    }

    pub fn exists_pmq(&self, uid: usize) -> bool {
        return self.exists_pq(uid) || self.exists_mq(uid);
    }

    pub fn exists_pq(&self, uid: usize) -> bool {
        for content in &self.priority_queue {
            if content.reserved_by == None {
                return true;
            }
        }
        return false;
    }

    pub fn exists_mq(&self, uid: usize) -> bool {
        for content in &self.main_queue {
            if content.reserved_by == None {
                return true;
            }
        }
        return false;
    }

    pub fn get_content_by_uid(&self, uid: usize) -> Option<(Content, QueueType)> {
        for content in &self.priority_queue {
            if content.uid == uid {
                return Some((content.clone(), QueueType::PriorityQueue));
            }
        }

        for content in &self.main_queue {
            if content.uid == uid {
                return Some((content.clone(), QueueType::MainQueue));
            }
        }

        return None;
    }

    fn remove_from_queue_by_uid(&mut self, uid: usize, queue_type: QueueType) -> Option<Content> {
        let temp;
        match queue_type {
            QueueType::PriorityQueue => {
                temp = self.get_index_by_uid_in_queue(uid, QueueType::PriorityQueue);
                if temp.is_some() {
                    self.priority_queue.remove(temp.unwrap());
                }
            }
            QueueType::MainQueue => {
                temp = self.get_index_by_uid_in_queue(uid, QueueType::MainQueue);
                self.priority_queue.remove(temp.unwrap());
            }
            QueueType::All => {
                let pq = self.get_index_by_uid_in_queue(uid, QueueType::PriorityQueue);
                if pq.is_some() {
                    self.priority_queue.remove(pq.unwrap());
                }
                let mq = self.get_index_by_uid_in_queue(uid, QueueType::MainQueue);
                if mq.is_some() {
                    self.priority_queue.remove(mq.unwrap());
                }
            }
        }
        return None;
    }

    pub fn prioritise_existing_encode(&mut self, uid: usize) {
        let main_queue = self.exists_mq(uid);
        let priority_queue = self.exists_pq(uid);

        if main_queue && !priority_queue {
            match self.get_content_by_uid(uid) {
                None => {
                    //nothing to do
                }
                Some((content, location)) => {
                    if content.reserved_by.is_none() {
                        let content = self.remove_from_queue_by_uid(uid, location);
                        if content.is_some() {
                            self.priority_queue.push_back(content.unwrap());
                        }
                    }
                }
            }
        }
    }

    pub fn prioritise_content_by_content(&mut self, content: Content) {
        let mut exists = false;
        if !self.exists_pq(content.uid) {
            self.priority_queue.push_back(content);
        }
    }

    /* pub fn prioritise_content_by_uids(&mut self, uids: Vec<usize>) {
        while uids.len() > 0 {
            let mut exists = false;
            let current_uid = uids.
            if !exists
        }


            if found {
                self.priority_queue.push(self.main_queue.remove(index));
            }
    } */
}
