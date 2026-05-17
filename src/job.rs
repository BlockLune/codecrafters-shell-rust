use std::fmt::Display;
use std::process::Child;

pub enum JobIndicator {
    Current,
    Previous,
    None,
}

impl Display for JobIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobIndicator::Current => write!(f, "+"),
            JobIndicator::Previous => write!(f, "-"),
            JobIndicator::None => write!(f, " "),
        }
    }
}

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

    pub fn reap(&mut self) -> String {
        match self.child.try_wait() {
            Ok(Some(_)) => String::from("Done"),
            Ok(None) => String::from("Running"),
            Err(_) => String::from("Unknown"),
        }
    }

    pub fn compute_job_status(jobs: &mut [Job]) -> Vec<Option<(JobIndicator, String)>> {
        let active_indices: Vec<usize> = jobs
            .iter()
            .enumerate()
            .filter(|(_, job)| !job.done)
            .map(|(i, _)| i)
            .collect();

        let current_idx = active_indices.iter().last().copied();
        let previous_idx = active_indices.iter().rev().nth(1).copied();

        jobs.iter_mut().enumerate().map(|(i, job)| {
            if job.done {
                return None;
            }

            let indicator = if Some(i) == current_idx {
                JobIndicator::Current
            } else if Some(i) == previous_idx {
                JobIndicator::Previous
            } else {
                JobIndicator::None
            };

            let status = job.reap();

            Some((indicator, status))

        }).collect()
    }
}
