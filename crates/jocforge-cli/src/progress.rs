// pattern: Imperative Shell
//! Reports long-running CLI work without changing any processing decisions.

use std::{
    io::{self, IsTerminal, Write},
    time::Instant,
};

pub(crate) struct ProgressBar {
    label: &'static str,
    total: Option<u64>,
    next_report: u64,
    last_reported: Option<u64>,
    terminal: bool,
    line_open: bool,
    started_at: Instant,
}

impl ProgressBar {
    pub(crate) fn new(label: &'static str) -> Self {
        let terminal = io::stderr().is_terminal();
        Self {
            label,
            total: None,
            next_report: 0,
            last_reported: None,
            terminal,
            line_open: false,
            started_at: Instant::now(),
        }
    }

    pub(crate) fn update(&mut self, completed: u64, total: u64) {
        if self.total.is_none() {
            self.total = Some(total);
            self.next_report = (total / 100).max(1);
        }
        if self.total != Some(total) || total == 0 {
            return;
        }

        let completed = completed.min(total);
        let should_report = self.terminal
            || self.last_reported.is_none()
            || completed >= self.next_report
            || completed == total;
        if !should_report || self.last_reported == Some(completed) {
            return;
        }

        self.render(completed, total);
        self.last_reported = Some(completed);
        if !self.terminal {
            self.next_report = completed.saturating_add((total / 100).max(1));
        }
    }

    pub(crate) fn finish(&mut self) {
        if let Some(total) = self.total {
            self.update(total, total);
        }
        if self.terminal && self.line_open {
            let mut stderr = io::stderr().lock();
            let _ = writeln!(stderr);
            let _ = stderr.flush();
            self.line_open = false;
        }
    }

    fn render(&mut self, completed: u64, total: u64) {
        let percentage = (u128::from(completed) * 100 / u128::from(total)) as u64;
        let width = 24_u64;
        let filled = percentage * width / 100;
        let mut bar = String::with_capacity(width as usize);
        for position in 0..width {
            bar.push(if position < filled { '#' } else { '-' });
        }
        let elapsed = self.started_at.elapsed().as_secs();
        let line = format!(
            "{} [{}] {:>3}% ({}/{}) elapsed {:02}:{:02}",
            self.label,
            bar,
            percentage,
            completed,
            total,
            elapsed / 60,
            elapsed % 60,
        );
        let mut stderr = io::stderr().lock();
        if self.terminal {
            let _ = write!(stderr, "\r{line}");
            self.line_open = true;
        } else {
            let _ = writeln!(stderr, "{line}");
        }
        let _ = stderr.flush();
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.terminal && self.line_open {
            let mut stderr = io::stderr().lock();
            let _ = writeln!(stderr);
            let _ = stderr.flush();
        }
    }
}
