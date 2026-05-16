pub struct Job {
    pub job_number: usize, // start from 1
    pub command_line: String,
    pub pid: u32,
}

impl Job {
    pub fn new(
        job_number: usize,
        command_line: &str,
        pid: u32,
    ) -> Self {
        Job {
            job_number,
            command_line: command_line.to_string(),
            pid,
        }
    }
}
