use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EPOCH_MS: i64 = 1735689600000; // 2025-01-01T00:00:00Z
const WORKER_BITS: i64 = 10;
const SEQUENCE_BITS: i64 = 12;
const MAX_SEQUENCE: i64 = (1 << SEQUENCE_BITS) - 1; // 4095

pub struct Snowflake {
    worker_id: i64,
    sequence: AtomicI64,
    last_timestamp: AtomicI64,
}

impl Snowflake {
    pub fn new(worker_id: i64) -> Self {
        let max_worker = (1 << WORKER_BITS) - 1;
        assert!(
            worker_id <= max_worker,
            "worker_id must be <= {}",
            max_worker
        );
        Self {
            worker_id,
            sequence: AtomicI64::new(0),
            last_timestamp: AtomicI64::new(0),
        }
    }

    pub fn next(&self) -> i64 {
        loop {
            let mut now = now_ms();

            loop {
                let last = self.last_timestamp.load(Ordering::Acquire);

                if now < last {
                    now = now_ms();
                    continue;
                }

                if now > last {
                    self.sequence.store(0, Ordering::Release);
                    match self.last_timestamp.compare_exchange(
                        last,
                        now,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    ) {
                        Ok(_) => break,
                        Err(_) => {
                            now = now_ms();
                            continue;
                        }
                    }
                }

                // now == last
                let seq = self.sequence.fetch_add(1, Ordering::AcqRel);
                if seq < MAX_SEQUENCE {
                    let id = ((now - EPOCH_MS) << (WORKER_BITS + SEQUENCE_BITS))
                        | (self.worker_id << SEQUENCE_BITS)
                        | seq;
                    return id;
                }

                // sequence overflow, wait for next ms
                now = now_ms();
                while now <= self.last_timestamp.load(Ordering::Acquire) {
                    now = now_ms();
                    std::hint::spin_loop();
                }
            }
        }
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_unique_ids() {
        let sf = Snowflake::new(1);
        let mut ids = Vec::new();
        for _ in 0..1000 {
            ids.push(sf.next());
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn ids_are_monotonic() {
        let sf = Snowflake::new(0);
        let mut prev = sf.next();
        for _ in 0..1000 {
            let next = sf.next();
            assert!(
                next > prev,
                "not monotonic: {} <= {}",
                next,
                prev
            );
            prev = next;
        }
    }

    #[test]
    fn ids_are_positive() {
        let sf = Snowflake::new(1);
        for _ in 0..100 {
            assert!(sf.next() > 0);
        }
    }
}
