#![feature(const_mut_refs)]

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
    const GROUPS: [usize; 6] = [4; 6];

    const SCHEDULER: schedule_solver::Schedule =
        schedule_solver::Schedule::new(&GROUPS, GROUPS.len());
    const BUF: [usize; SCHEDULER.get_block_size()] = {
        let mut buf = [0; SCHEDULER.get_block_size()];
        if !SCHEDULER.initialise_buffer(&mut buf) {
            [1][1]; // Means failure
        }
        buf
    };
    println!("scheduler: {:?}", SCHEDULER);
    println!("buf: {:?}", BUF);

    let mut buf = BUF.to_vec();

    let mut current_depth = 0;

    let mut total_step_count = 0;
    let mut highest_players_placed = 0;
    let mut lowest_empty_tables = usize::MAX;
    let mut check_frequency = 1;
    let mut check_counter = 0;
    let mut i = 0;
    let mut last_print = std::time::Instant::now();

    let mut recent_depth_stats = (0, usize::MAX);
    loop {
        {
            let target_size = (current_depth + 2) * SCHEDULER.get_block_size();
            if target_size > buf.len() {
                buf.resize(target_size, 0);
            }
            let buffer: &mut [usize] = &mut buf[current_depth * SCHEDULER.get_block_size()..];
            let (buf_1, buf_2) = buffer.split_at_mut(SCHEDULER.get_block_size());
            if let Some(finished) = SCHEDULER.step(buf_1, buf_2) {
                if finished {
                    assert_eq!(SCHEDULER.get_players_placed(buf_1), 4 * 6 * 6);
                    assert_eq!(SCHEDULER.get_empty_table_count(buf_1), 0);
                    println!("Found a solution: {:?}", buf_1);
                    //current_depth -= 1;
                    return;
                } else {
                    assert_ne!(buf_1, buf_2);
                    assert!(current_depth <= SCHEDULER.get_players_placed(buf_2));
                    current_depth += 1;
                }
            } else {
                current_depth = current_depth.checked_sub(1).expect("No valid schedule");
            }
        }
        total_step_count += 1;

        {
            let players_placed =
                SCHEDULER.get_players_placed(&buf[current_depth * SCHEDULER.get_block_size()..]);
            let empty_tables =
                SCHEDULER.get_empty_table_count(&buf[current_depth * SCHEDULER.get_block_size()..]);
            let mut should_print = false;
            if players_placed >= highest_players_placed {
                highest_players_placed = players_placed;
                should_print = true;
            }
            if empty_tables < lowest_empty_tables {
                lowest_empty_tables = empty_tables;
                should_print = true;
            }
            if should_print {
                println!(
                    "New best: {:?} with depth {} step {} empty tables {}",
                    players_placed, current_depth, total_step_count, empty_tables
                );
                let mut output = String::new();
                SCHEDULER
                    .format_schedule(
                        &buf[current_depth * SCHEDULER.get_block_size()..],
                        &mut output,
                    )
                    .unwrap();
                println!("Schedule:\n{}", output);
            }

            if current_depth > recent_depth_stats.0 {
                recent_depth_stats.0 = current_depth;
            }

            if current_depth < recent_depth_stats.1 {
                recent_depth_stats.1 = current_depth;
            }

            i += 1;
            check_counter += 1;
            if check_counter > check_frequency {
                check_counter = 0;
                if last_print.elapsed().as_millis() > 400 {
                    println!(
                    "Current depth {} (recent min {} max {}) (best players_placed {} lowest empty_tables {}) with rate {}/s",
                    current_depth,
					recent_depth_stats.1,
					recent_depth_stats.0,
                    highest_players_placed,
					lowest_empty_tables,
                    i as f64 / last_print.elapsed().as_secs_f64()
                );

                    recent_depth_stats = (0, usize::MAX);

                    last_print = std::time::Instant::now();
                    check_frequency = (i / 16).max(1);
                    i = 0;
                }
            }
        }
    }

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
                    best_length = size;

                    println!("Ones: {:?}", scheduler.count_ones());

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
        }
    }
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = output_thread.join();
}
