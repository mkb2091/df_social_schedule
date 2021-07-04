use core::num::NonZeroUsize;

#[derive(Debug)]
pub enum ScheduleErrors {
    ZeroLengthGroups,
    PlayerCountOverflow,
    TooSmallBuffer,
    RoundsTooLarge,
}

#[derive(Debug)]
pub enum ScheduleResult {}

#[derive(Debug)]
struct Offsets {
    players_placed_counter_offset: usize,
    played_with_offset: usize,
    played_on_table_total_offset: usize,
    played_in_round_offset: usize,
    played_on_table_offset: usize,
    played_on_table_size: usize,
    block_size: usize,
}

impl Offsets {
    const fn new(
        played_with_size: usize,
        played_on_table_total_size: usize,
        played_in_round_size: usize,
        played_on_table_size: usize,
    ) -> Self {
        let players_placed_counter_offset = 0;
        let played_with_offset = 1;
        let played_on_table_total_offset = played_with_offset + played_with_size;
        let played_in_round_offset = played_on_table_total_offset + played_on_table_total_size;
        let played_on_table_offset = played_in_round_offset + played_in_round_size;
        let block_size = played_on_table_offset + played_on_table_size;
        Self {
            players_placed_counter_offset,
            played_with_offset,
            played_on_table_total_offset,
            played_in_round_offset,
            played_on_table_offset,
            played_on_table_size,
            block_size,
        }
    }
}

#[derive(Debug)]
pub struct Schedule<'a> {
    tables: &'a [usize],
    rounds: usize,
    player_count: usize,
    player_bit_word_count: usize,
    offsets: Offsets,
}

impl<'a> Schedule<'a> {
    pub const fn new(tables: &'a [usize], rounds: usize) -> Self {
        let mut player_count: usize = 0;
        let mut i = 0;
        while i < tables.len() {
            player_count += tables[i];
            i += 1;
        }
        let player_bit_word_count =
            player_count / Self::word_size() + (player_count % Self::word_size() != 0) as usize;

        let played_with_size = player_bit_word_count * player_count;
        let played_on_table_total_size = player_bit_word_count * tables.len();
        let played_in_round_size = player_bit_word_count * rounds;
        let played_on_table_size = player_bit_word_count * tables.len() * rounds;
        let offsets = Offsets::new(
            played_with_size,
            played_on_table_total_size,
            played_in_round_size,
            played_on_table_size,
        );

        Self {
            tables,
            player_count,
            rounds,
            player_bit_word_count,
            offsets,
        }
    }

    pub const fn get_block_size(&self) -> usize {
        self.offsets.block_size
    }

    #[must_use]
    pub const fn initialise_buffer(&self, buffer: &mut [usize]) -> Option<()> {
        if buffer.len() < self.offsets.block_size {
            return None;
        }
        let mut i = 0;
        while i < self.offsets.block_size {
            buffer[i] = 0;
            i += 1;
        }
        let max = Self::get_byte_and_mask(self.player_count);
        let start =
            self.offsets.played_on_table_offset + self.player_bit_word_count * self.tables.len(); // Skip first round
        let end = self.offsets.played_on_table_offset + self.offsets.played_on_table_size;
        let mut i = 0;
        while start + i < end {
            let current_byte = i % self.player_bit_word_count;
            buffer[start + i] = if current_byte > 0 {
                0
            } else if current_byte == max.0 {
                max.1 - 1
            } else {
                usize::MAX
            };
            i += 1;
        }

        let mut pos = 0;
        let mut table_number = 0;
        while table_number < self.tables.len() {
            let size = self.tables[table_number];
            let mut player = pos;
            while player < pos + size {
                self.apply_player(buffer, 0, table_number, player);
                player += 1;
            }
            pos += size;
            table_number += 1;
        }

        None
    }

    const fn word_size() -> usize {
        core::mem::size_of::<usize>() * 8
    }
    const fn get_byte_and_mask(player: usize) -> (usize, usize) {
        let byte = player / Self::word_size();
        let mask = 1 << (player - (byte * Self::word_size()));
        (byte, mask)
    }

    const fn apply_player(
        &self,
        buffer: &mut [usize],
        round: usize,
        table: usize,
        player: usize,
    ) -> Option<()> {
        if round >= self.rounds || table >= self.tables.len() || player >= self.player_count {
            return None;
        }
        let (byte, player_mask) = Self::get_byte_and_mask(player);
        let remove_player_mask = !player_mask;
        let offset = self.offsets.played_on_table_offset;
        buffer[self.offsets.players_placed_counter_offset] += 1;
        {
            let mut r2 = 0;
            while r2 < self.rounds {
                // Remove player from the table in other rounds
                buffer[offset
                    + self.player_bit_word_count * (r2 * self.tables.len() + table)
                    + byte] &= remove_player_mask;
                r2 += 1;
            }
        }
        {
            let mut t2 = 0;
            while t2 < self.tables.len() {
                // Remove player from other tables in the same round
                buffer[offset
                    + self.player_bit_word_count * (round * self.tables.len() + t2)
                    + byte] &= remove_player_mask;
                t2 += 1;
            }
        }
        // Add player to played in round
        buffer[self.offsets.played_in_round_offset + self.player_bit_word_count * round + byte] |=
            player_mask;
        // Add player to played on table
        buffer[self.offsets.played_on_table_total_offset
            + self.player_bit_word_count * table
            + byte] |= player_mask;

        {
            let mut other_byte = 0;
            while other_byte < self.player_bit_word_count {
                let mut other_players = buffer[offset
                    + self.player_bit_word_count * (round * self.tables.len() + table)
                    + other_byte];
                // Add other players to players played with
                buffer[self.offsets.played_with_offset
                    + self.player_bit_word_count * player
                    + other_byte] |= other_players;
                while other_players != 0 {
                    let trailing_zeros = other_players.trailing_zeros() as usize;
                    let other_player = other_byte * Self::word_size() + trailing_zeros;
                    let other_player_bit = 1 << trailing_zeros;
                    other_players &= !other_player_bit;

                    // Add player to other players played with
                    buffer[self.offsets.played_with_offset
                        + self.player_bit_word_count * other_player
                        + byte] |= player_mask;
                }
                other_byte += 1;
            }
        }

        if table == 0 {
            let mask_less = player_mask - 1;
            let mask_above = !player_mask & !mask_less;
            {
                let mut r2 = 0;
                while r2 < round {
                    {
                        let mut b2 = 0;
                        while b2 < byte {
                            buffer[offset
                                + self.player_bit_word_count * (r2 * self.tables.len() + table)
                                + b2] &= 0;
                            b2 += 1;
                        }
                    }
                    buffer[offset
                        + self.player_bit_word_count * (r2 * self.tables.len() + table)
                        + byte] &= mask_less;
                    r2 += 1;
                }
            }
            {
                let mut r2 = round + 1;
                while r2 < self.rounds {
                    buffer[offset
                        + self.player_bit_word_count * (r2 * self.tables.len() + table)
                        + byte] &= mask_above;
                    {
                        let mut b2 = byte + 1;
                        while b2 < self.player_bit_word_count {
                            buffer[offset
                                + self.player_bit_word_count * (r2 * self.tables.len() + table)
                                + b2] &= 0;
                            b2 += 1;
                        }
                    }
                    r2 += 1;
                }
            }
        }

        // Add player back to their own table+round
        buffer[offset + self.player_bit_word_count * (round * self.tables.len() + table) + byte] |=
            player_mask;
        Some(())
    }

    pub const fn get_players_placed(&self, buffer: &[usize]) -> usize {
        buffer[self.offsets.players_placed_counter_offset]
    }

    pub fn step(&self, buffer_1: &mut [usize], buffer_2: &mut [usize]) -> Option<bool> {
        let buffer_1 = &mut buffer_1[..self.offsets.block_size];
        let buffer_2 = &mut buffer_2[..self.offsets.block_size];

        let offset = self.offsets.played_on_table_offset;
        for round in 0..self.rounds {
            for (table, table_size) in self.tables.iter().enumerate() {
                let mut fixed_player_count = 0;
                let mut potential_player_count = 0;
                for byte in 0..self.player_bit_word_count {
                    let fixed = buffer_1[self.offsets.played_in_round_offset
                        + self.player_bit_word_count * round
                        + byte]
                        & buffer_1[self.offsets.played_on_table_total_offset
                            + self.player_bit_word_count * table
                            + byte];
                    fixed_player_count += fixed.count_ones();
                    potential_player_count += (buffer_1[offset
                        + self.player_bit_word_count * (round * self.tables.len() + table)
                        + byte]
                        & !(fixed))
                        .count_ones();
                }

                if fixed_player_count < *table_size as u32 {
                    if potential_player_count < *table_size as u32 {
                        return None;
                    }
                    for byte in 0..self.player_bit_word_count {
                        let mut potential = buffer_1[offset
                            + self.player_bit_word_count * (round * self.tables.len() + table)
                            + byte];
                        potential &= !buffer_1[self.offsets.played_in_round_offset
                            + self.player_bit_word_count * round
                            + byte];
                        potential &= !buffer_1[self.offsets.played_on_table_total_offset
                            + self.player_bit_word_count * table
                            + byte];
                        while potential != 0 {
                            let trailing_zeros = potential.trailing_zeros() as usize;
                            let player = byte * Self::word_size() + trailing_zeros;
                            let player_bit = 1 << trailing_zeros;
                            potential &= !player_bit;
                            buffer_2.copy_from_slice(buffer_1);
                            buffer_1[offset
                                + self.player_bit_word_count
                                    * (round * self.tables.len() + table)
                                + byte] &= !player_bit;
                            self.apply_player(buffer_2, round, table, player);
                            return Some(false);
                        }
                    }
                }
            }
        }
        Some(true)
    }
}
