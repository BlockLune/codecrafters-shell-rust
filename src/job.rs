use std::process::Child;

pub struct Job {
    pub job_number: usize, // start from 1
    pub command_line: String,
    pub child: Child,
    pub done: bool,
}

impl Job {
    pub fn new(job_number: usize, command_line: &str, child: Child) -> Self {
        Job {
            job_number,
            command_line: command_line.to_string(),
            child,
            done: false,
        }
    }
}
