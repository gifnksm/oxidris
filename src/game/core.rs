use crate::{
    field::Field,
    mino::{Mino, MinoGenerator, MinoKind},
};

const SCORE_TABLE: [usize; 5] = [0, 1, 5, 25, 100];

#[derive(Debug, Clone)]
pub(crate) struct GameCore {
    field: Field,
    falling_mino: Mino,
    held_mino: Option<MinoKind>,
    hold_used: bool,
    mino_generator: MinoGenerator,
    score: usize,
    cleared_lines: usize,
}

impl GameCore {
    pub(crate) fn new() -> Self {
        let first_mino = MinoKind::I; // dummy initial value
        let mut game = Self {
            field: Field::INITIAL,
            falling_mino: Mino::new(first_mino),
            held_mino: None,
            hold_used: false,
            mino_generator: MinoGenerator::new(),
            score: 0,
            cleared_lines: 0,
        };
        game.begin_next_mino_fall();
        game
    }

    pub(crate) fn level(&self) -> usize {
        self.cleared_lines / 10
    }

    pub(crate) fn cleared_lines(&self) -> usize {
        self.cleared_lines
    }

    pub(crate) fn score(&self) -> usize {
        self.score
    }

    pub(crate) fn field(&self) -> &Field {
        &self.field
    }

    pub(crate) fn falling_mino(&self) -> &Mino {
        &self.falling_mino
    }

    pub(crate) fn set_falling_mino(&mut self, mino: Mino) -> Result<(), ()> {
        if self.field.is_colliding(&mino) {
            return Err(());
        }
        self.falling_mino = mino;
        Ok(())
    }

    pub(crate) fn set_falling_mino_unchecked(&mut self, mino: Mino) {
        self.falling_mino = mino;
    }

    pub(crate) fn held_mino(&self) -> Option<MinoKind> {
        self.held_mino
    }

    pub(crate) fn next_minos(&self) -> impl Iterator<Item = MinoKind> + '_ {
        self.mino_generator.next_minos()
    }

    pub(crate) fn simulate_drop_position(&self) -> Mino {
        let mut dropped = self.falling_mino;
        while let Some(mino) = dropped.down() {
            if self.field.is_colliding(&mino) {
                break;
            }
            dropped = mino;
        }
        dropped
    }

    pub(crate) fn try_hold(&mut self) -> Result<(), ()> {
        if self.hold_used {
            return Err(());
        }
        if let Some(held_mino) = self.held_mino {
            let mino = Mino::new(held_mino);
            if self.field.is_colliding(&mino) {
                return Err(());
            }
            self.held_mino = Some(self.falling_mino.kind());
            self.falling_mino = mino;
        } else {
            self.held_mino = Some(self.falling_mino.kind());
            self.begin_next_mino_fall();
        }
        self.hold_used = true;
        Ok(())
    }

    pub(crate) fn complete_mino_drop(&mut self) -> Result<(), ()> {
        self.field.fill_mino(&self.falling_mino);
        let line = self.field.clear_lines();
        self.score += SCORE_TABLE[line];
        self.cleared_lines += line;

        self.begin_next_mino_fall();
        if self.field.is_colliding(&self.falling_mino) {
            return Err(());
        }

        self.hold_used = false;
        Ok(())
    }

    fn begin_next_mino_fall(&mut self) {
        self.falling_mino = Mino::new(self.mino_generator.pop_next());
    }
}
