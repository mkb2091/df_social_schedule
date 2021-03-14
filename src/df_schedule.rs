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
    on_current_table_offset: usize,
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
        let temp_buffer = vec![T::MAX; player_bit_word_count].into_boxed_slice();

        Self {
            groups: groups.to_vec().into_boxed_slice(),
            player_count,
            player_bit_word_count,
            players_played_with,
            schedule: Vec::new(),
            played_on_table_total,
            played_in_round,
            on_current_table,
            on_current_table_offset: 0,
            current_table: 0,
            current_position_in_table: 0,
            current_round: 0,
            min_player: None,
            temp_buffer,
            best_length: 0,
        }
    }

    pub fn print_schedule(&mut self) -> String {
        let mut output = String::new();
        let mut round_vec = Vec::new();
        let mut table_vec = Vec::new();
        for player in self.schedule.iter() {
            table_vec.push(player);
            if table_vec.len() >= self.groups[round_vec.len()] {
                round_vec.push(table_vec);
                table_vec = Vec::new();
                if round_vec.len() >= self.groups.len() {
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

    #[inline(always)]
    fn generate_potential_players(&mut self) {
        for (i, ptr) in self.temp_buffer.iter_mut().enumerate() {
            *ptr = !self.played_in_round[self.current_round * self.player_bit_word_count + i]
                & !self.played_on_table_total[self.current_table * self.player_bit_word_count + i]
                & !self.on_current_table[self.on_current_table_offset + i];
            for other_player in
                self.schedule[self.schedule.len() - self.current_position_in_table..].iter()
            {
                *ptr &= !self.players_played_with[other_player * self.player_bit_word_count + i];
            }
        }
    }

    #[inline(always)]
    pub fn step(&mut self) -> Option<Option<usize>> {
        'outer: for (i, mut temp) in self.temp_buffer.iter().cloned().enumerate() {
            while temp != T::ZERO {
                let trailing_zeros = temp.trailing_zeros() as usize;
                let player = trailing_zeros + i * T::SIZE;
                let player_bit = T::ONE << trailing_zeros;
                temp &= !player_bit;
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
                self.on_current_table[self.on_current_table_offset + i] |= player_bit;

                self.schedule.push(player);

                self.current_position_in_table += 1;
                if self.current_position_in_table >= self.groups[self.current_table] {
                    let round_offset = self.current_round * self.player_bit_word_count;
                    let table_offset = self.current_table * self.player_bit_word_count;
                    for (i, block) in self.on_current_table[self.on_current_table_offset..]
                        .iter()
                        .enumerate()
                    {
                        self.played_in_round[round_offset + i] |= *block;
                        self.played_on_table_total[table_offset + i] |= *block;
                    }
                    for player in
                        self.schedule[self.schedule.len() - self.current_position_in_table..].iter()
                    {
                        for i in 0..self.player_bit_word_count {
                            self.players_played_with[player * self.player_bit_word_count + i] |=
                                self.on_current_table[self.on_current_table_offset + i];
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

                    self.on_current_table_offset += self.player_bit_word_count;
                    self.on_current_table.resize(
                        self.on_current_table_offset + self.player_bit_word_count,
                        T::ZERO,
                    );
                    self.min_player = None;

                    for (i, ptr) in self.temp_buffer.iter_mut().enumerate() {
                        *ptr = !self.played_in_round
                            [self.current_round * self.player_bit_word_count + i]
                            & !self.played_on_table_total
                                [self.current_table * self.player_bit_word_count + i];
                    }
                } else {
                    self.min_player = Some(player);
                    for (i, ptr) in self.temp_buffer.iter_mut().enumerate() {
                        *ptr &= !self.players_played_with[player * self.player_bit_word_count + i];
                    }
                }
                return Some(Some(self.schedule.len()));
            }
        }
		if self.schedule.len() < self.player_count {
			return None;
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
                self.on_current_table_offset -= self.player_bit_word_count;
                self.on_current_table
                    .truncate(self.on_current_table_offset + self.player_bit_word_count);
            } else {
                self.current_position_in_table -= 1;
            }

            let byte = player / T::SIZE;

            self.played_on_table_total[self.current_table * self.player_bit_word_count + byte] &=
                !(T::ONE << (player - (byte * T::SIZE)));
            self.played_in_round[self.current_round * self.player_bit_word_count + byte] &=
                !(T::ONE << (player - (byte * T::SIZE)));
            self.on_current_table[self.on_current_table_offset + byte] &=
                !(T::ONE << (player - (byte * T::SIZE)));
            for other_player in
                self.schedule[self.schedule.len() - self.current_position_in_table..].iter()
            {
                let other_byte = other_player / T::SIZE;
                self.players_played_with[player * self.player_bit_word_count + other_byte] &=
                    !(T::ONE << (other_player - (other_byte * T::SIZE)));
                self.players_played_with[other_player * self.player_bit_word_count + byte] &=
                    !(T::ONE << (player - (byte * T::SIZE)));
            }

            self.generate_potential_players();

            Some(None)
        } else {
            None
        }
    }
}
