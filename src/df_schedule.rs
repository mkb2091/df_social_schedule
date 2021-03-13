use std::ops::*;

pub trait Word:
    Sized
    + BitAnd<Self, Output = Self>
    + BitAndAssign<Self>
    + BitOr<Self, Output = Self>
    + BitOrAssign<Self>
    + BitXor<Self, Output = Self>
    + BitXorAssign<Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Not<Output = Self>
    + Copy
    + Clone
    + std::fmt::Debug
    + std::fmt::Binary
    + Eq
{
    const SIZE: usize = 8 * std::mem::size_of::<Self>();
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    fn count_ones(self) -> u32;
    fn count_zeros(self) -> u32;
    fn leading_ones(self) -> u32;
    fn leading_zeros(self) -> u32;
    fn trailing_ones(self) -> u32;
    fn trailing_zeros(self) -> u32;
}

impl Word for u8 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline(always)]
    fn count_zeros(self) -> u32 {
        self.count_zeros()
    }
    #[inline(always)]
    fn leading_ones(self) -> u32 {
        self.leading_ones()
    }
    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        self.trailing_ones()
    }
    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
    }
}
impl Word for u16 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline(always)]
    fn count_zeros(self) -> u32 {
        self.count_zeros()
    }
    #[inline(always)]
    fn leading_ones(self) -> u32 {
        self.leading_ones()
    }
    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        self.trailing_ones()
    }
    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
    }
}
impl Word for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline(always)]
    fn count_zeros(self) -> u32 {
        self.count_zeros()
    }
    #[inline(always)]
    fn leading_ones(self) -> u32 {
        self.leading_ones()
    }
    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        self.trailing_ones()
    }
    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
    }
}
impl Word for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline(always)]
    fn count_zeros(self) -> u32 {
        self.count_zeros()
    }
    #[inline(always)]
    fn leading_ones(self) -> u32 {
        self.leading_ones()
    }
    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        self.trailing_ones()
    }
    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
    }
}
impl Word for u128 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline(always)]
    fn count_zeros(self) -> u32 {
        self.count_zeros()
    }
    #[inline(always)]
    fn leading_ones(self) -> u32 {
        self.leading_ones()
    }
    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        self.trailing_ones()
    }
    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
    }
}
impl Word for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const MAX: Self = Self::MAX;
    #[inline(always)]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline(always)]
    fn count_zeros(self) -> u32 {
        self.count_zeros()
    }
    #[inline(always)]
    fn leading_ones(self) -> u32 {
        self.leading_ones()
    }
    #[inline(always)]
    fn leading_zeros(self) -> u32 {
        self.leading_zeros()
    }
    #[inline(always)]
    fn trailing_ones(self) -> u32 {
        self.trailing_ones()
    }
    #[inline(always)]
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
    }
}

#[derive(Debug)]
pub struct DFScheduler<T>
where
    T: Word,
{
    groups: Box<[usize]>,
    player_count: usize,
    player_bit_word_count: usize,
    players_played_with: Box<[T]>,
    schedule: Vec<usize>,
    played_on_table_total: Box<[T]>,
    played_in_round: Vec<T>,
    on_current_table: Vec<T>,
    current_table: usize,
    current_position_in_table: usize,
    current_round: usize,
    min_player: Option<usize>,
    temp_buffer: Box<[T]>,
    best_length: usize,
}

impl<T: Word> DFScheduler<T> {
    pub fn new(groups: &[usize]) -> Self {
        let player_count = groups.iter().sum();
        let player_bit_word_count = player_count / T::SIZE + (player_count % T::SIZE != 0) as usize;
        let players_played_with =
            vec![T::ZERO; player_bit_word_count * player_count].into_boxed_slice();
        let played_on_table_total =
            vec![T::ZERO; player_bit_word_count * groups.len()].into_boxed_slice();
        let played_in_round = vec![T::ZERO; player_bit_word_count];
        let on_current_table = vec![T::ZERO; player_bit_word_count];
        let temp_buffer = vec![T::ZERO; player_bit_word_count].into_boxed_slice();

        Self {
            groups: groups.to_vec().into_boxed_slice(),
            player_count,
            player_bit_word_count,
            players_played_with,
            schedule: Vec::new(),
            played_on_table_total,
            played_in_round,
            on_current_table,
            current_table: 0,
            current_position_in_table: 0,
            current_round: 0,
            min_player: None,
            temp_buffer,
            best_length: 0,
        }
    }

    pub fn print_schedule(&mut self) {
        if self.schedule.len() > self.best_length {
            self.best_length = self.schedule.len();
        } else {
            return;
        }

        println!("Schedule");
        let mut round_vec = Vec::new();
        let mut table_vec = Vec::new();
        for player in self.schedule.iter() {
            table_vec.push(player);
            if table_vec.len() >= self.groups[round_vec.len()] {
                round_vec.push(table_vec);
                table_vec = Vec::new();
                if round_vec.len() >= self.groups.len() {
                    println!("{:?}", round_vec);
                    round_vec = Vec::new()
                }
            }
        }
        if !table_vec.is_empty() {
            round_vec.push(table_vec);
        }
        if !round_vec.is_empty() {
            println!("{:?}", round_vec);
        }
    }

    pub fn step(&mut self) -> bool {
        self.temp_buffer.fill(T::MAX);
        for i in 0..self.player_bit_word_count {
            self.temp_buffer[i] = !self.played_in_round
                [self.current_round * self.player_bit_word_count + i]
                & !self.played_on_table_total[self.current_table * self.player_bit_word_count + i]
                & !self.on_current_table
                    [self.on_current_table.len() - self.player_bit_word_count + i];
            for other_player in
                self.schedule[self.schedule.len() - self.current_position_in_table..].iter()
            {
                self.temp_buffer[i] &=
                    !self.players_played_with[other_player * self.player_bit_word_count + i];
            }
        }

        'outer: for (i, mut temp) in self.temp_buffer.iter().cloned().enumerate() {
            while temp != T::ZERO {
                let trailing_zeros = temp.trailing_zeros() as usize;
                let player_bit = T::ONE << trailing_zeros;
                temp &= !player_bit;
                let player = trailing_zeros + i * T::SIZE;
                if player >= self.player_count {
                    break 'outer;
                }
                if self
                    .min_player
                    .map(|min_player| player <= min_player)
                    .unwrap_or(false)
                {
                    continue;
                }
                let len = self.on_current_table.len();
                self.on_current_table[len - self.player_bit_word_count + i] |= player_bit;

                self.schedule.push(player);

                

                self.current_position_in_table += 1;
                if self.current_position_in_table >= self.groups[self.current_table] {
                    for i in 0..self.player_bit_word_count {
                        self.played_in_round
                            [self.current_round * self.player_bit_word_count + i] |= self
                            .on_current_table
                            [self.on_current_table.len() - self.player_bit_word_count + i];
                        self.played_on_table_total
                            [self.current_table * self.player_bit_word_count + i] |= self
                            .on_current_table
                            [self.on_current_table.len() - self.player_bit_word_count + i];
                    }
                    for player in
                        self.schedule[self.schedule.len() - self.current_position_in_table..].iter()
                    {
                        for i in 0..self.player_bit_word_count {
                            self.players_played_with[player * self.player_bit_word_count + i] |=
                                self.on_current_table
                                    [self.on_current_table.len() - self.player_bit_word_count + i];
                        }
                    }

                    self.current_table += 1;
                    if self.current_table >= self.groups.len() {
                        self.current_table = 0;
                        self.current_round += 1;
                        self.played_in_round.resize(
                            (self.current_round + 1) * self.player_bit_word_count,
                            T::ZERO,
                        );
                    }
                    self.current_position_in_table = 0;

                    let len = self.on_current_table.len();
                    self.on_current_table
                        .resize(len + self.player_bit_word_count, T::ZERO);
                    self.min_player = None;
                } else {
                    self.min_player = Some(player);
                }

                self.print_schedule();
                return true;
            }
        }
        self.min_player = self.schedule.pop();

        if let Some(player) = self.min_player {
            if self.current_position_in_table == 0 {
                if self.current_table == 0 {
                    self.current_table = self.groups.len() - 1;
                    assert!(self.current_round != 0);

                    self.current_round -= 1;
                    self.played_in_round
                        .truncate((self.current_round + 1) * self.player_bit_word_count);
                } else {
                    self.current_table -= 1;
                }
                self.current_position_in_table = self.groups[self.current_table] - 1;
                let len = self.on_current_table.len();
                assert!(len > self.player_bit_word_count);
                self.on_current_table
                    .truncate(len - self.player_bit_word_count);
            } else {
                self.current_position_in_table -= 1;
            }

            let byte = player / T::SIZE;
            self.played_on_table_total[self.current_table * self.player_bit_word_count + byte] &=
                !(T::ONE << (player - (byte * T::SIZE)));
            self.played_in_round[self.current_round * self.player_bit_word_count + byte] &=
                !(T::ONE << (player - (byte * T::SIZE)));
            let len = self.on_current_table.len();
            self.on_current_table[len - self.player_bit_word_count + byte] &=
                !(T::ONE << (player - (byte * T::SIZE)));
            
            
            for other_player in self.schedule[self.schedule.len() - self.current_position_in_table..].iter() {
                let other_byte = other_player / T::SIZE;
                self.players_played_with[player * self.player_bit_word_count + other_byte] &= !(T::ONE << (other_player - (other_byte * T::SIZE)));
                self.players_played_with[other_player * self.player_bit_word_count + byte] &= !(T::ONE << (player - (byte * T::SIZE)));
            }
        }

        self.min_player.is_some()
    }
}
