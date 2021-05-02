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
    + Sub<Self, Output = Self>
    + SubAssign<Self>
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

macro_rules! derive_word {
    ($x: ty) => {
        impl Word for $x {
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
    };
}

derive_word!(u8);
derive_word!(u16);
derive_word!(u32);
derive_word!(u64);
derive_word!(u128);
derive_word!(usize);

#[derive(Debug)]
pub struct DFScheduler<T>
where
    T: Word,
{
    groups: Box<[std::num::NonZeroUsize]>,
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
    pub fn new(groups: &[std::num::NonZeroUsize]) -> Self {
        let player_count = groups.iter().map(|x| x.get()).sum();
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

    pub fn clone_schedule_into(&self, other: &mut Vec<usize>) {
        other.clone_from(&self.schedule);
    }

    pub fn get_schedule(&self) -> Vec<usize> {
        self.schedule.clone()
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
        let (min_player_byte, min_player_mask) = if let Some(min_player) = self.min_player {
            let min_player_byte = min_player / T::SIZE;
            let min_player_bit = (T::ONE << (min_player - (min_player_byte * T::SIZE)));
            let min_player_mask = !((min_player_bit - T::ONE) | min_player_bit);
            assert!(min_player_mask != T::MAX);
            (Some(min_player_byte), min_player_mask)
        } else {
            (None, T::ZERO)
        };
        'outer: for (i, mut temp) in self.temp_buffer.iter().cloned().enumerate() {
            if let Some(min_player_byte) = min_player_byte {
                if i < min_player_byte {
                    continue;
                }
                if i == min_player_byte {
                    temp &= min_player_mask;
                }
            }
            while temp != T::ZERO {
                let trailing_zeros = temp.trailing_zeros() as usize;
                let player = trailing_zeros + i * T::SIZE;
                let player_bit = T::ONE << trailing_zeros;
                temp &= !player_bit;
                if player >= self.player_count {
                    break 'outer;
                }
                debug_assert!(self
                    .min_player
                    .map(|min_player| player >= min_player)
                    .unwrap_or(true));
                self.on_current_table[self.on_current_table_offset + i] |= player_bit;

                self.schedule.push(player);

                self.current_position_in_table += 1;
                if self.current_position_in_table >= self.groups[self.current_table].get() {
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

                    self.min_player = None;

                    self.current_table += 1;
                    if self.current_table >= self.groups.len() {
                        assert!(self.schedule.len() % self.player_count == 0);
                        self.min_player =
                            Some(self.schedule[self.schedule.len() - self.player_count]); // Each row must start with higher player than previous row
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
                if self.schedule.len() > self.best_length {
                    self.best_length = self.schedule.len();
                }
                return Some(Some(self.schedule.len()));
            }
        }

        if self.backtrack() {
            Some(None)
        } else {
            None
        }
    }

    fn backtrack(&mut self) -> bool {
        if self.schedule.len() < self.player_count {
            return false;
        }
        self.min_player = self.schedule.pop();

        if let Some(player) = self.min_player {
            let byte = player / T::SIZE;

            let mask = !(T::ONE << (player - (byte * T::SIZE)));

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
                self.current_position_in_table = self.groups[self.current_table].get() - 1;
                self.on_current_table_offset -= self.player_bit_word_count;
                self.on_current_table
                    .truncate(self.on_current_table_offset + self.player_bit_word_count);
            } else {
                self.current_position_in_table -= 1;
            }

            self.played_in_round[self.current_round * self.player_bit_word_count + byte] &= mask;
            self.on_current_table[self.on_current_table_offset + byte] &= mask;
            self.played_on_table_total[self.current_table * self.player_bit_word_count + byte] &=
                mask;

            for other_player in
                self.schedule[self.schedule.len() - self.current_position_in_table..].iter()
            {
                let other_byte = other_player / T::SIZE;
                self.players_played_with[player * self.player_bit_word_count + other_byte] &=
                    !(T::ONE << (other_player - (other_byte * T::SIZE)));
                self.players_played_with[other_player * self.player_bit_word_count + byte] &= mask;
            }

            self.generate_potential_players();
            true
        } else {
            false
        }
    }
}
