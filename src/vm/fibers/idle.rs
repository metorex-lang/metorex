//! The pause taken while every thread waits on something outside the
//! interpreter, and what ends that pause early. A signal arriving during
//! the pause ends it, since `poll` returns when one interrupts it.

use super::*;
use std::os::fd::RawFd;
use std::time::{Duration, Instant};

/// The longest a pause lasts. A wait that names neither a descriptor nor a
/// deadline, such as one on a child process or on a C function running on a
/// thread of its own, is looked at again this often.
const LONGEST_PAUSE: Duration = Duration::from_millis(10);

/// The first pause after a round in which a thread ran, which doubles with
/// each round in which none did, up to `LONGEST_PAUSE`.
const FIRST_PAUSE: Duration = Duration::from_millis(1);

impl VirtualMachine {
    /// Name a descriptor the running thread waits on, so the pause that
    /// follows a round in which no thread ran ends as soon as it is ready.
    pub(crate) fn note_waited_descriptor(&mut self, descriptor: RawFd, writing: bool) {
        let events = if writing { libc::POLLOUT } else { libc::POLLIN };
        // Turns given without a pause, as `Thread.pass` gives them, name the
        // same descriptors again each time round.
        if descriptor < 0
            || self
                .idle_descriptors
                .iter()
                .any(|entry| entry.fd == descriptor && entry.events == events)
        {
            return;
        }
        self.idle_descriptors.push(libc::pollfd {
            fd: descriptor,
            events,
            revents: 0,
        });
    }

    /// Name when the running thread's wait runs out, so the pause that
    /// follows ends by then.
    pub(crate) fn note_wake_deadline(&mut self, deadline: Instant) {
        self.idle_deadline = Some(match self.idle_deadline {
            Some(earlier) => earlier.min(deadline),
            None => deadline,
        });
    }

    /// A round in which some thread ran: the next pause starts short, and
    /// what this round's waits named is named again by the next.
    pub(crate) fn note_busy_round(&mut self) {
        self.idle_rounds = 0;
        self.idle_descriptors.clear();
        self.idle_deadline = None;
    }

    /// Pause after a round in which no thread ran, until a descriptor a
    /// waiting thread named is ready, a deadline one named or a
    /// `Timeout.timeout` limit comes, a signal arrives, or the pause has
    /// lasted as long as rounds like this one allow.
    pub(crate) fn pause_while_idle(&mut self) {
        let doublings = self.idle_rounds.min(4);
        self.idle_rounds = self.idle_rounds.saturating_add(1);
        let mut pause = (FIRST_PAUSE * (1u32 << doublings)).min(LONGEST_PAUSE);
        let now = Instant::now();
        let limits = self.timeout_limits.iter().map(|(deadline, _, _)| *deadline);
        if let Some(deadline) = self.idle_deadline.take().into_iter().chain(limits).min() {
            pause = pause.min(deadline.saturating_duration_since(now));
        }
        // With nothing to watch, the pause is a park, which a function run
        // without the interpreter ends early by waking this thread.
        if self.idle_descriptors.is_empty() {
            std::thread::park_timeout(pause);
            return;
        }
        let mut watched = std::mem::take(&mut self.idle_descriptors);
        let milliseconds = pause.as_micros().div_ceil(1000) as libc::c_int;
        // SAFETY: `watched` holds as many entries as the count given, and
        // lives for the call.
        let ready = unsafe {
            libc::poll(
                watched.as_mut_ptr(),
                watched.len() as libc::nfds_t,
                milliseconds,
            )
        };
        if ready > 0 {
            self.idle_rounds = 0;
        }
        watched.clear();
        self.idle_descriptors = watched;
    }

    /// Wait until a descriptor in `reading` has something to read or one in
    /// `writing` has room to write, or `limit` passes, raising EBADF for one
    /// that is not open. Signal handlers run meanwhile, a `Timeout.timeout`
    /// limit ends the wait by raising, and every other thread takes its
    /// turns.
    pub(crate) fn wait_for_descriptors(
        &mut self,
        reading: &[RawFd],
        writing: &[RawFd],
        limit: Option<Duration>,
        position: Position,
    ) -> Result<(), MetorexError> {
        let deadline = limit.map(|length| Instant::now() + length);
        let mut asked: Vec<libc::pollfd> = reading
            .iter()
            .map(|&descriptor| (descriptor, libc::POLLIN))
            .chain(
                writing
                    .iter()
                    .map(|&descriptor| (descriptor, libc::POLLOUT)),
            )
            .map(|(fd, events)| libc::pollfd {
                fd,
                events,
                revents: 0,
            })
            .collect();
        loop {
            // SAFETY: `asked` holds as many entries as the count given, and
            // the wait of zero asks without waiting.
            let ready = unsafe { libc::poll(asked.as_mut_ptr(), asked.len() as libc::nfds_t, 0) };
            if asked
                .iter()
                .any(|entry| entry.revents & libc::POLLNVAL != 0)
            {
                return Err(crate::vm::errors::simple_exception(
                    "Errno::EBADF",
                    "Bad file descriptor",
                    position,
                ));
            }
            if ready != 0 {
                return Ok(());
            }
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Ok(());
            }
            self.deliver_pending_signals(position)?;
            self.raise_expired_timeout(position)?;
            for entry in &asked {
                self.note_waited_descriptor(entry.fd, entry.events == libc::POLLOUT);
            }
            if let Some(deadline) = deadline {
                self.note_wake_deadline(deadline);
            }
            if self.other_threads_are_waiting() {
                self.wait_for_other_threads(position);
                self.raise_if_thread_killed(position)?;
            } else {
                self.pause_while_idle();
            }
        }
    }
}
