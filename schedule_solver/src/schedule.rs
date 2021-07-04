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
    empty_table_count_offset: usize,
    to_explore_offset: usize,
    played_with_offset: usize,
    played_on_table_total_offset: usize,
    played_in_round_offset: usize,
    played_on_table_offset: usize,
    potential_on_table_offset: usize,
    played_on_table_size: usize,
    block_size: usize,
}

impl Offsets {
    const fn new(
        to_explore_size: usize,
        played_with_size: usize,
        played_on_table_total_size: usize,
        played_in_round_size: usize,
        played_on_table_size: usize,
    ) -> Self {
        let players_placed_counter_offset = 0;
        let empty_table_count_offset = players_placed_counter_offset + 1;
        let to_explore_offset = empty_table_count_offset + 1;
        let played_with_offset = to_explore_offset + to_explore_size;
        let played_on_table_total_offset = played_with_offset + played_with_size;
        let played_in_round_offset = played_on_table_total_offset + played_on_table_total_size;
        let played_on_table_offset = played_in_round_offset + played_in_round_size;
        let potential_on_table_offset = played_on_table_offset + played_on_table_size;
        let block_size = potential_on_table_offset + played_on_table_size;
        Self {
            players_placed_counter_offset,
            empty_table_count_offset,
            to_explore_offset,
            played_with_offset,
            played_on_table_total_offset,
            played_in_round_offset,
            played_on_table_offset,
            potential_on_table_offset,
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
        let to_explore_size = rounds * tables.len() * 2;
        let offsets = Offsets::new(
            to_explore_size,
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
    pub const fn initialise_buffer(&self, buffer: &mut [usize]) -> bool {
        if buffer.len() < self.offsets.block_size {
            return false;
        }
        let mut i = 0;
        while i < self.offsets.block_size {
            buffer[i] = 0;
            i += 1;
        }

        let max = Self::get_byte_and_mask(self.player_count);
        let start =
            self.offsets.potential_on_table_offset + self.player_bit_word_count * self.tables.len(); // Skip first round
        let end = self.offsets.potential_on_table_offset + self.offsets.played_on_table_size;
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

        buffer[self.offsets.empty_table_count_offset] = (self.rounds - 1) * self.tables.len();
        let mut round = 1;
        while round < self.rounds {
            let mut table = 0;
            while table < self.tables.len() {
                let offset =
                    self.offsets.to_explore_offset + ((round - 1) * self.tables.len() + table) * 2;
                buffer[offset] = round;
                buffer[offset + 1] = table;
                table += 1;
            }
            round += 1;
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
        true
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
        buffer[self.offsets.players_placed_counter_offset] += 1; // Will double count if called multiple times
        {
            let mut r2 = 0;
            while r2 < self.rounds {
                // Remove player from the table in other rounds
                buffer[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count * (r2 * self.tables.len() + table)
                    + byte] &= remove_player_mask;
                r2 += 1;
            }
        }
        {
            let mut t2 = 0;
            while t2 < self.tables.len() {
                // Remove player from other tables in the same round
                buffer[self.offsets.potential_on_table_offset
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
                let mut other_players = buffer[self.offsets.played_on_table_offset
                    + self.player_bit_word_count * (round * self.tables.len() + table)
                    + other_byte];

                //buffer[self.offsets.potential_on_table_offset
                //    + self.player_bit_word_count * (round * self.tables.len() + table)
                //    + other_byte] &= !other_players;

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

        // Add player to their own table+round
        buffer[self.offsets.potential_on_table_offset
            + self.player_bit_word_count * (round * self.tables.len() + table)
            + byte] |= player_mask;
        buffer[self.offsets.played_on_table_offset
            + self.player_bit_word_count * (round * self.tables.len() + table)
            + byte] |= player_mask;
        Some(())
    }

    pub const fn get_players_placed(&self, buffer: &[usize]) -> usize {
        buffer[self.offsets.players_placed_counter_offset]
    }

    pub const fn get_empty_table_count(&self, buffer: &[usize]) -> usize {
        buffer[self.offsets.empty_table_count_offset]
    }

    pub fn step(&self, buffer_1: &mut [usize], buffer_2: &mut [usize]) -> Option<bool> {
        let buffer_1 = &mut buffer_1[..self.offsets.block_size];
        let buffer_2 = &mut buffer_2[..self.offsets.block_size];

        let offset = self.offsets.potential_on_table_offset;

        for i in 0..buffer_1[self.offsets.empty_table_count_offset] {
            let round = buffer_1[self.offsets.to_explore_offset + i * 2];
            let table = buffer_1[self.offsets.to_explore_offset + i * 2 + 1];
            let table_size = self.tables[table];

            let mut fixed_player_count = 0;
            for byte in 0..self.player_bit_word_count {
                let fixed = buffer_1[self.offsets.played_on_table_offset
                    + self.player_bit_word_count * (round * self.tables.len() + table)
                    + byte];
                fixed_player_count += fixed.count_ones();
            }

            if fixed_player_count < table_size as u32 {
                for byte in 0..self.player_bit_word_count {
                    let fixed = buffer_1[self.offsets.played_on_table_offset
                        + self.player_bit_word_count * (round * self.tables.len() + table)
                        + byte];
                    let potential = buffer_1[offset
                        + self.player_bit_word_count * (round * self.tables.len() + table)
                        + byte]
                        & !fixed;
                    let mut temp = potential;
                    while temp != 0 {
                        let trailing_zeros = temp.trailing_zeros() as usize;
                        let player = byte * Self::word_size() + trailing_zeros;
                        let player_bit = 1 << trailing_zeros;
                        temp &= !player_bit;

                        if buffer_1[self.offsets.played_with_offset
                            + self.player_bit_word_count * player
                            + byte]
                            & fixed
                            != 0
                        {
                            buffer_1[offset
                                + self.player_bit_word_count
                                    * (round * self.tables.len() + table)
                                + byte] &= !player_bit;
                            continue;
                        }

                        buffer_2.copy_from_slice(buffer_1);
                        buffer_1[offset
                            + self.player_bit_word_count * (round * self.tables.len() + table)
                            + byte] &= !player_bit;
                        self.apply_player(buffer_2, round, table, player);
                        if fixed_player_count + 1 == table_size as u32 {
                            let pos = self.offsets.to_explore_offset + i * 2;
                            let end = self.offsets.to_explore_offset
                                + buffer_2[self.offsets.empty_table_count_offset] * 2;
                            buffer_2[pos..end].rotate_left(2);
                            //buffer_2.swap(pos, end - 2);
                            //buffer_2.swap(pos + 1, end - 1);
                            buffer_2[self.offsets.empty_table_count_offset] -= 1;
                        }
                        return Some(false);
                    }
                }
                return None; // Could not place any player but fixed_player_count < table_size
            } else if fixed_player_count >= table_size as u32 {
                unreachable!();
            }
        }
        Some(true)
    }
}
