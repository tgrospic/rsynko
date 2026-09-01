# rsynko-ratatui

This crate interprets the manager as a screen a reader moves around in. It owns terminal mechanics and nothing else: what a page states, which actions have meaning, what a transfer would change, and when a run may start are stated by the specification crates and never decided here.

## What it interprets

`RatatuiScreen` interprets `rsynko-ui`'s screen vocabulary — lines, panes, emphasis, a share drawn as a bar — as ratatui widgets. Which widget, which colour, which border, how finely a bar draws its share, and how a cursor is placed in a line of text are this crate's decisions, and the vocabulary states none of them.

Three interpreters of `rsynko-session` attend to what is running, one per kind of run: downloads this program retrieves itself, transfers another program performs, and inspections that read what a source offers. Each states only what beginning, reading, holding, and ending a run of its kind is; the order those things happen in is one pass of `SessionExt::attend`, which the specification derives. A fourth, `Monotonic`, interprets `ClockAlg` as the machine's own clock.

Input is the same shape in reverse: a crossterm key event is decoded into a `rsynko_ui::Keystroke`, and what that keystroke denotes is the manager's to say. A key the vocabulary does not bind denotes nothing, so nothing here has to know which keys mean what.

## Leaving

Two things about leaving are this interpreter's own, because they are about the terminal rather than about the manager.

`Ctrl+C` is bound to nothing in the vocabulary, and it is the key people press out of habit to stop whatever a terminal is running. Pressed once it states what it is for and ends nothing; pressed twice in quick succession it states safe exit, exactly as the bound `Ctrl+Q` does. The manager hears nothing of a single one.

Once leaving is stated, every run is told to stop and the reader is held here while they stop — a transfer is another program and would otherwise be left running, and a download is a thread of this one that states what it does for as long as it does it. A run that will not stop does not keep the terminal: after a short grace the reader is out regardless.

## Entry point

[`terminal::run`] takes the terminal over, states the requests it was given, attends to everything running, paints what the manager denotes, reads what the reader does, and gives the terminal back. It is handed requests rather than a command line, because reading a command line is not a terminal mechanism — [`rsynko`](https://docs.rs/rsynko) does that, and chooses this interpreter or the line-oriented one.

The walkthrough of what the screens actually state — every page, every field, every action — is in [`rsynko_manager`], where the meaning lives.
