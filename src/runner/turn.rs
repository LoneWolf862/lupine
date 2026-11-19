//! A turn's game consequences and output share one commit boundary.
use super::*;

pub(super) enum Error {
    Rule(String),
    Io(io::Error),
}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Rule(e)
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub(super) fn finish(
    world: &World,
    encounters: &EncounterContent,
    state: &mut GameState,
    rng: &mut EncounterRng,
    mut step: crate::events::Step,
    output: &mut impl Write,
) -> Result<(), Error> {
    let spent = step.eligible
        && (step.performed || *state != step.before)
        && step.before.events.ending.is_none();
    crate::systems::prune_equipment(state);
    if spent && state.events.ending.is_none() {
        let modifiers = state.active_enemies.clone();
        let (lines, defeated) = crate::systems::tick(world, state, encounters);
        for line in lines {
            writeln!(output, "{line}\n")?;
        }
        if !defeated.is_empty() {
            encounters.record_defeat(state);
        }
        for enemy in defeated {
            step.triggers
                .push(crate::events::Trigger::Defeat(enemy.enemy_id.clone()));
            let reward_room = state.current_room.clone();
            writeln!(
                output,
                "{}\n",
                combat::award_defeat(
                    encounters,
                    world,
                    world.systems.progression.as_ref(),
                    state,
                    &enemy,
                    rng
                )?
            )?;
            sync_object_room(&reward_room, world, state, encounters, rng, output)?;
        }
        if world.systems.combat.retaliation == crate::systems::Retaliation::EveryAction
            || (world.systems.combat.retaliation == crate::systems::Retaliation::Combat
                && step.combat_turn)
        {
            let mut active = state.active_enemies.clone();
            for enemy in &mut active {
                if let Some(old) = modifiers
                    .iter()
                    .find(|e| e.instance_id == enemy.instance_id)
                {
                    enemy.statuses = old.statuses.clone();
                }
            }
            let text = combat::retaliate(world, encounters, state, &active, rng);
            if !text.is_empty() {
                writeln!(output, "{text}\n")?;
            }
        }
        if is_defeated(state) && !is_defeated(&step.before) {
            writeln!(
                output,
                "YOU HAVE BEEN DEFEATED\nAvailable commands: saves, load <name>, quit.\n"
            )?;
        }
    }
    let old_room = state.current_room.clone();
    for text in crate::events::finish(world, state, step)? {
        writeln!(output, "{text}\n")?;
    }
    sync_object_room(&old_room, world, state, encounters, rng, output)?;
    Ok(())
}

/// Interactive preparation is live. Once mutation starts, even flush cannot
/// publish success text until the complete turn has committed.
pub(super) struct Output<W> {
    inner: W,
    staged: Option<Vec<u8>>,
}
impl<W: Write> Output<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            staged: None,
        }
    }
    pub fn stage(&mut self) {
        if self.staged.is_none() {
            self.staged = Some(Vec::new());
        }
    }
    pub fn discard(&mut self) {
        self.staged = None;
    }
    pub fn commit(&mut self) -> io::Result<()> {
        if let Some(bytes) = self.staged.take() {
            self.inner.write_all(&bytes)?;
        }
        Ok(())
    }
}
impl<W: Write> Write for Output<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match &mut self.staged {
            Some(buffer) => {
                buffer.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            None => self.inner.write(bytes),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
