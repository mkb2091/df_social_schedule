pub fn print_schedule(schedule: &[usize], groups: &[std::num::NonZeroUsize]) -> String {
    let mut output = String::new();
    let mut round_vec = Vec::new();
    let mut table_vec = Vec::new();
    for player in schedule.iter() {
        table_vec.push(player);
        if table_vec.len() >= groups[round_vec.len()].get() {
            round_vec.push(table_vec);
            table_vec = Vec::new();
            if round_vec.len() >= groups.len() {
                output.push_str(&format!("{:?}\n", round_vec));
                round_vec.clear();
            }
        }
    }
    if !table_vec.is_empty() {
        round_vec.push(table_vec);
    }
    if !round_vec.is_empty() {
        output.push_str(&format!("{:?}", round_vec));
    }
    output
}

fn main() {
    let groups = [4; 6]
        .iter()
        .filter_map(|x| std::num::NonZeroUsize::new(*x))
        .collect::<Vec<_>>();
    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<usize>::new(&groups);
    let mut best_length = 0;
    let mut best_opp_count = 0;

    let ops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let best_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let best_string = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let current_string = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let output_thread = {
        let ops = ops.clone();
        let best_counter = best_counter.clone();
        let running = running.clone();
        let best_string = best_string.clone();
        let current_string = current_string.clone();
        let groups = groups.clone();
        std::thread::spawn(move || {
            let now = std::time::Instant::now();
            let mut should_continue = true;
            while should_continue {
                std::thread::sleep(std::time::Duration::from_millis(300));
                println!(
                    "ops /s: {}\nCurrent:\n{}\nBest:\n{}\nBest Count: {}\nBest Count/s: {}\n",
                    ops.load(std::sync::atomic::Ordering::Relaxed) as f32
                        / now.elapsed().as_secs_f32(),
                    print_schedule(&current_string.lock().unwrap(), &groups),
                    best_string.lock().unwrap(),
                    best_counter.load(std::sync::atomic::Ordering::Relaxed),
                    best_counter.load(std::sync::atomic::Ordering::Relaxed) as f32
                        / now.elapsed().as_secs_f32()
                );
                should_continue = running.load(std::sync::atomic::Ordering::Relaxed);
            }
        })
    };
    let mut local_ops = 0;
    let mut cloned_scheduler = scheduler.clone();
    while let Some(size) = scheduler.step() {
        if let Some(size) = size {
            local_ops += 1;
            if local_ops > 100_000 {
                ops.fetch_add(local_ops, std::sync::atomic::Ordering::Relaxed);
                current_string
                    .lock()
                    .unwrap()
                    .clone_from(scheduler.get_schedule());
                local_ops = 0;
            }
            if size >= best_length {
                cloned_scheduler.clone_from(&scheduler);
                cloned_scheduler.fill();
                let opp_count = cloned_scheduler.get_unique_opponents();
                if opp_count > best_opp_count {
                    best_opp_count = opp_count;

                    let mut temp_best_string = print_schedule(&scheduler.get_schedule(), &groups);
                    temp_best_string.push_str("\n\n");
                    temp_best_string
                        .push_str(&print_schedule(&cloned_scheduler.get_schedule(), &groups));
                    temp_best_string.push_str(&format!("\nUnique Opponent Count: {:?}", opp_count));
                    *best_string.lock().unwrap() = temp_best_string;
                    best_counter.store(1, std::sync::atomic::Ordering::Relaxed);
                } else if opp_count == best_opp_count {
                    best_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }

            if size > best_length {
                best_length = size;
            }
        }
    }
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = output_thread.join();
}
