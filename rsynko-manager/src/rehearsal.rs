use crate::*;
use alux_sdk::trait_algebra;
use ambassador::delegatable_trait;

/// Specifies the rehearsal mode of one request.
///
/// Some requests can state what they would do instead of doing it. A request that cannot states
/// no mode at all, so a renderer never offers a rehearsal where the meaning does not exist.
#[delegatable_trait]
pub trait QueueDryRunAlg: ManagerSorts {
    /// Turns the rehearsal mode of one request on or off.
    fn set_queue_dry_run(&mut self, id: Self::Id, dry_run: bool);
}

/// Specifies rehearsal requests and external observations.
#[delegatable_trait]
pub trait RehearsalStateAlg: ManagerSorts {
    /// Applies one rehearsal observation.
    fn apply_rehearsal_event(&mut self, id: Self::Id, event: RehearsalObservationOp<Self::Change>);
}

/// Specifies what a rehearsal stated one request would do.
pub trait RehearsalViewAlg {
    /// Represents one change a rehearsal states.
    type Change;

    /// Observes what rehearsal has stated about this request.
    fn rehearsal(&self) -> RehearsalState<'_>;

    /// Observes the changes a rehearsal stated, in the order it stated them.
    fn planned_changes(&self) -> impl Iterator<Item = &Self::Change>;
}

/// Specifies what one planned change states about itself.
pub trait PlannedChangeAlg {
    /// Observes the path the change names, relative to the transferred folder.
    fn change_path(&self) -> &str;
    /// Observes what would happen to that path.
    fn change_kind(&self) -> ChangeKind;
    /// Observes the byte count the change moves, when the rehearsal stated one.
    fn change_size(&self) -> Option<u64>;
}

/// Defines the first-order observation stream of one rehearsal.
///
/// An observation is a method, so the reified stream is generated from this vocabulary rather than
/// written out. An interpreter states what each observation *means* and never restates the shape.
#[trait_algebra(derive(Clone, Debug, PartialEq, Eq))]
pub trait RehearsalObservation {
    /// Represents one change a rehearsal states.
    type Change;

    /// Denotes that an interpreter started rehearsing the request.
    fn started(&self);

    /// Supplies everything the rehearsal stated would happen, in its own order.
    fn reported(&self, changes: Vec<Self::Change>);

    /// Denotes rehearsal failure.
    fn failed(&self, message: String);
}

/// Denotes what rehearsal has stated about one request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RehearsalState<'a> {
    /// Denotes a request no one has rehearsed.
    Unrehearsed,
    /// Denotes a rehearsal an interpreter is currently performing.
    Rehearsing,
    /// Denotes a rehearsal that stated what would happen.
    Reported,
    /// Denotes a rehearsal that failed, and why.
    Failed(&'a str),
}

/// Denotes what one rehearsed change would do to a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    /// Denotes a path the transfer would create.
    Create,
    /// Denotes a path the transfer would replace with newer content.
    Update,
    /// Denotes a path the transfer would remove.
    Delete,
    /// Denotes a path the transfer would leave exactly as it is.
    Unchanged,
}

impl ChangeKind {
    /// States every kind a rehearsal distinguishes, the changes before what stays.
    pub const REPORTED: [Self; 4] = [Self::Create, Self::Update, Self::Delete, Self::Unchanged];

    /// Observes whether the transfer would alter the path.
    #[must_use]
    pub const fn alters(self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}
