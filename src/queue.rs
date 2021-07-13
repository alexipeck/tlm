use crate::content::Job;
use std::collections::VecDeque;

pub enum QueueType {
    MainQueue,
    PriorityQueue,
    All,
}

pub struct Queue {
    pub priority_queue: VecDeque<Job>,
    pub main_queue: VecDeque<Job>,
    cache_directories: VecDeque<String>,
}

impl Queue {
    pub fn new(cache_directories: VecDeque<String>) -> Queue {
        Queue {
            priority_queue: VecDeque::new(),
            main_queue: VecDeque::new(),
            cache_directories: cache_directories,
        }
    }

    pub fn fill_cache_by_uid(&mut self, uid: usize) {
        let mut done = false;
        for job in &mut self.priority_queue {
            if job.uid == uid && self.cache_directories.len() > 0{
                job.cache_directory = Some(self.cache_directories[0].clone());
                done = true;
            }
        }

        if !done {
            for job in &mut self.main_queue {
                if job.uid == uid && self.cache_directories.len() > 0{
                    job.cache_directory = Some(self.cache_directories[0].clone());
                }
            }
        }
    }

    pub fn exists_pmq(&self, uid: usize) -> bool {
        return self.exists_pq(uid) || self.exists_mq(uid);
    }

    pub fn exists_pq(&self, uid: usize) -> bool {
        for job in &self.priority_queue {
            if job.uid == uid {
                return true;
            }
        }
        return false;
    }

    pub fn exists_mq(&self, uid: usize) -> bool {
        for job in &self.main_queue {
            if job.uid == uid {
                return true;
            }
        }
        return false;
    }
    
    pub fn add_job_to_queue(&mut self, job: Job) {
        //if the job isn't already in the queue
        if !self.exists_pmq(job.uid) {
            self.main_queue.push_back(job);
        }
    }

    pub fn add_job_to_priority_queue(&mut self, job: Job) {
        //if the job isn't already in the queue
        if !self.exists_pmq(job.uid) {
            self.priority_queue.push_back(job);
        }      
    }

    pub fn remove_from_queue_by_uid(&mut self, job_uid: usize) -> Option<Job> {
        for (index, job) in self.priority_queue.iter().enumerate() {
            if job.uid == job_uid {
                return self.priority_queue.remove(index);
            }
        }
        
        for (index, job) in self.main_queue.iter().enumerate() {
            if job.uid == job_uid {
                return self.main_queue.remove(index);
            }
        }

        return None;
    }

    pub fn handle_by_uid(&mut self, job_uid: usize, operator: String) {
        let mut delete: bool = false;
        
        for job in &mut self.priority_queue {
            if job.uid == job_uid {
                job.handle(operator.clone());
                delete = true;
            }
        }
        for job in &mut self.main_queue {
            if job.uid == job_uid {
                job.handle(operator.clone());
                delete = true;
            }
        }
        if delete {
            self.remove_from_queue_by_uid(job_uid);
        }
    }

    pub fn run_job(&mut self, operator: String) {
        //currently encodes first unreserved Job
        let mut uid_to_handle: Option<usize> = None;
        for job in &self.priority_queue {
            if job.reserved_by.is_none() {
                uid_to_handle = Some(job.uid);
                break;
            }
        }
        for job in &self.main_queue {
            if job.reserved_by.is_none() {
                uid_to_handle = Some(job.uid);
                break;
            }
        }
        if uid_to_handle.is_some() {
            self.handle_by_uid(uid_to_handle.unwrap(), operator);
        }
    }



    pub fn prioritise_existing_job(&mut self, job_uid: usize) {
        if self.exists_mq(job_uid) {
            let temp = self.remove_from_queue_by_uid(job_uid);
            if temp.is_some() {
                self.priority_queue.push_back(temp.unwrap());
            }
        }

        if self.exists_pq(job_uid) {
            let temp = self.remove_from_queue_by_uid(job_uid);
            if temp.is_some() {
                self.priority_queue.push_front(temp.unwrap());
            }
        }
    }

    pub fn prioritise_existing_jobs(&mut self, job_uids: Vec<usize>) {
        for job_uid in job_uids {
            self.prioritise_existing_job(job_uid);
        }
    }

    pub fn print(&mut self) {
        for job in &self.priority_queue {
            job.print("pq");
        }

        for job in &self.main_queue {
            job.print("mq");
        }
    }

    pub fn get_full_queue_length(&mut self) -> usize {
        return self.priority_queue.len() + self.main_queue.len();
    }

    pub fn get_next_unreserved(&mut self, operator: String) -> Option<usize> {
        for job in &mut self.priority_queue {
            if job.reserved_by == None {
                job.reserve(operator);
                return Some(job.uid);
            }
        }

        for job in &mut self.main_queue {
            if job.reserved_by == None {
                job.reserve(operator);
                return Some(job.uid);
            }
        }

        return None;
    }
}
