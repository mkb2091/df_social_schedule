fn main() {
    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u32>::new(&[4, 4, 4, 4, 4, 4]);
    while scheduler.step() {}
}
