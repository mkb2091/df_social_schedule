fn main() {
    let groups = [4, 4, 4, 4, 4, 4]
        .iter()
        .filter_map(|x| std::num::NonZeroUsize::new(*x))
        .collect::<Vec<_>>();
    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u32>::new(&groups);
    let mut best_length = 0;

    let ops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let best_string = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let current_string = std::sync::Arc::new(std::sync::Mutex::new(String::new()));

    let output_thread = {
        let ops = ops.clone();
        let running = running.clone();
        let best_string = best_string.clone();
        let current_string = current_string.clone();
        std::thread::spawn(move || {
            let now = std::time::Instant::now();
            let mut should_continue = true;
            while should_continue {
                std::thread::sleep(std::time::Duration::from_millis(300));
                println!(
                    "ops /s: {}\nCurrent:\n{}\nBest:\n{}\n",
                    ops.load(std::sync::atomic::Ordering::Relaxed) as f32
                        / now.elapsed().as_secs_f32(),
                    current_string.lock().unwrap(),
                    best_string.lock().unwrap()
                );
                should_continue = running.load(std::sync::atomic::Ordering::Relaxed);
            }
        })
    };
    let mut local_ops = 0;
    while let Some(size) = scheduler.step() {
        local_ops += 1;
        if local_ops > 100_000 {
            ops.fetch_add(local_ops, std::sync::atomic::Ordering::Relaxed);
            //*current_string.lock().unwrap() = scheduler.print_schedule();
            local_ops = 0;
        }

        if let Some(size) = size {
            if size > best_length {
                best_length = size;
                *best_string.lock().unwrap() = scheduler.print_schedule();
            }
        }
    }
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = output_thread.join();
}
