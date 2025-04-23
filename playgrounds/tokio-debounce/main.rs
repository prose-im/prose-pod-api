use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use debounce::DebouncedNotify;
use tokio::time::{sleep, Instant};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let notify = DebouncedNotify::new();
    let counter = Arc::new(AtomicU8::new(0));
    {
        let counter = counter.clone();
        notify.listen_debounced(Duration::from_millis(100), move |instant| {
            println!(
                "[RX] <- {:?}ms at {:?}ms",
                instant.duration_since(start).as_millis(),
                start.elapsed().as_millis(),
            );
            counter.fetch_add(1, Ordering::Relaxed);
        });
    }

    macro_rules! notify {
        () => {
            notify.notify();
            println!("[TX] -> {:?}ms", start.elapsed().as_millis());
        };
    }
    macro_rules! sleep {
        ($n:literal ms) => {
            sleep(Duration::from_millis($n)).await;
        };
    }
    macro_rules! expect_count {
        ($n:literal) => {
            assert_eq!(counter.load(Ordering::Relaxed), $n);
        };
    }

    println!("Start: {:?}ms", start.elapsed().as_millis());

    // Testing if events get emitted by default.
    sleep!(250 ms); // 250
    expect_count!(0);

    // Fast burst
    notify!();
    sleep!(10 ms); // 260
    notify!();
    sleep!(10 ms); // 270
    notify!();
    sleep!(10 ms); // 280
    notify!();
    sleep!(10 ms); // 290
    notify!();
    sleep!(10 ms); // 300
    notify!();

    // Wait
    expect_count!(0);
    sleep!(350 ms); // 650
    expect_count!(1);

    // Slow burst
    notify!();
    sleep!(70 ms); // 720
    notify!();
    sleep!(70 ms); // 790
    notify!();

    // Wait
    expect_count!(1);
    sleep!(110 ms); // 900
    expect_count!(2);

    // A single event
    notify!();

    // Wait
    expect_count!(2);
    sleep!(300 ms); // 1200
    expect_count!(3);

    println!("End: {:?}ms", start.elapsed().as_millis());
}

mod debounce {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Notify;
    use tokio::task::JoinHandle;
    use tokio::time::{sleep, Instant};

    pub struct DebouncedNotify {
        notify: Arc<Notify>,
    }

    impl DebouncedNotify {
        pub fn new() -> Self {
            Self {
                notify: Arc::new(Notify::new()),
            }
        }

        pub fn notify(&self) {
            self.notify.notify_waiters();
        }

        pub fn listen_debounced(
            &self,
            delay: Duration,
            callback: impl Fn(Instant) + Send + 'static,
        ) -> JoinHandle<()> {
            let notify = self.notify.clone();
            tokio::spawn(async move {
                let mut last_signal: Option<Instant> = None;
                loop {
                    tokio::select! {
                        _ = notify.notified() => {
                            last_signal = Some(Instant::now());
                        }
                        _ = sleep(delay), if last_signal.is_some_and(|i| i.elapsed() < delay) => {
                            callback(last_signal.unwrap());
                        }
                    }
                }
            })
        }
    }
}
