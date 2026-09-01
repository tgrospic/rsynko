//! Recovers Youtube's signature decipher program from the player script and applies it.
//!
//! The player states the decipher as one function over a character array, whose body is a sequence
//! of calls into a helper object holding exactly three primitive transformations: reversing the
//! array, dropping a prefix of it, and swapping the first character with one at an index. Those
//! primitives are what the whole program means, so recovering it is recovering that sequence.

use thiserror::Error;

/// Denotes failure to recover the decipher program from a player script.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SignatureProgramError {
    /// Denotes a script stating no decipher function.
    #[error("the player program states no signature decipher function")]
    MissingFunction,
    /// Denotes a decipher function whose helper object is absent.
    #[error("the decipher function calls a helper object the player program does not state")]
    MissingHelper,
    /// Denotes a helper call this interpreter does not recognize.
    #[error("the decipher function applies an unrecognized transformation: {0}")]
    UnknownTransformation(String),
}

/// Denotes one primitive transformation of the signature characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureStep {
    /// Reverses the remaining characters.
    Reverse,
    /// Drops the stated number of leading characters.
    Drop(usize),
    /// Exchanges the first character with the one at the stated index, modulo the length.
    Swap(usize),
}

/// Denotes the decipher program one player script states.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SignatureProgram {
    steps: Vec<SignatureStep>,
}

impl SignatureProgram {
    /// Recovers the decipher program stated by one player script.
    ///
    /// # Errors
    ///
    /// Returns the reason the script states no interpretable program.
    pub fn recover(player: &str) -> Result<Self, SignatureProgramError> {
        let body = decipher_body(player).ok_or(SignatureProgramError::MissingFunction)?;
        let helper = helper_name(body).ok_or(SignatureProgramError::MissingHelper)?;
        let object = helper_object(player, &helper).ok_or(SignatureProgramError::MissingHelper)?;
        body.split(';')
            .filter_map(|call| helper_call(call, &helper))
            .map(|(name, argument)| step(&object, name, argument))
            .collect::<Result<Vec<_>, _>>()
            .map(|steps| Self { steps })
    }

    /// Observes the transformations the program applies, in application order.
    #[must_use]
    pub fn steps(&self) -> &[SignatureStep] {
        &self.steps
    }

    /// Answers one signature challenge by applying every transformation in order.
    #[must_use]
    pub fn decipher(&self, signature: &str) -> String {
        let mut characters: Vec<char> = signature.chars().collect();
        for step in &self.steps {
            if characters.is_empty() {
                break;
            }
            match *step {
                SignatureStep::Reverse => characters.reverse(),
                SignatureStep::Drop(count) => {
                    characters.drain(..count.min(characters.len()));
                }
                SignatureStep::Swap(index) => {
                    let position = index % characters.len();
                    characters.swap(0, position);
                }
            }
        }
        characters.into_iter().collect()
    }
}

/// Observes the body of the function the player states over the signature characters.
fn decipher_body(player: &str) -> Option<&str> {
    // The function is recognized by what it does rather than by its name: it splits the signature
    // into characters and joins them back, and the player states no other function that does both.
    for marker in ["=a.split(\"\")", "=a.split('')"] {
        let mut search = player;
        let mut offset = 0;
        while let Some(index) = search.find(marker) {
            let start = offset + index + marker.len();
            let tail = &player[start..];
            if let Some(end) = tail.find("return a.join(") {
                let body = tail.get(..end)?;
                if !body.contains("function") {
                    return Some(body.trim_start_matches(';'));
                }
            }
            offset = start;
            search = &player[offset..];
        }
    }
    None
}

/// Names the helper object the decipher body applies.
fn helper_name(body: &str) -> Option<String> {
    let call = body
        .split(';')
        .map(str::trim)
        .find(|call| call.split_once('.').is_some_and(|(head, tail)| !head.is_empty() && tail.contains('(')))?;
    call.split('.').next().map(str::to_owned)
}

/// Observes the body of the helper object the player states under one name.
fn helper_object(player: &str, name: &str) -> Option<String> {
    // A player states the object as a `var` on its own or within a list, with or without spaces.
    let start = [format!("var {name}={{"), format!("var {name} = {{"), format!("{name}={{")]
        .into_iter()
        .find_map(|marker| Some(player.find(&marker)? + marker.len()))?;
    let tail = &player[start..];
    let mut depth = 1_u32;
    for (offset, character) in tail.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return tail.get(..offset).map(str::to_owned);
                }
            }
            _ => {}
        }
    }
    None
}

/// Reads one `helper.name(a,argument)` call of the decipher body.
fn helper_call<'a>(call: &'a str, helper: &str) -> Option<(&'a str, usize)> {
    let call = call.trim();
    let tail = call.strip_prefix(helper)?.strip_prefix('.')?;
    let (name, arguments) = tail.split_once('(')?;
    let argument = arguments.trim_end_matches(')').split(',').nth(1)?;
    Some((name, argument.trim().parse().unwrap_or(0)))
}

/// Classifies one helper function by the transformation its body performs.
fn step(object: &str, name: &str, argument: usize) -> Result<SignatureStep, SignatureProgramError> {
    let marker = format!("{name}:function");
    let start = object.find(&marker).ok_or_else(|| SignatureProgramError::UnknownTransformation(name.to_owned()))?;
    let body = &object[start..];
    let body = body.split_once('}').map_or(body, |(head, _)| head);
    if body.contains("reverse") {
        Ok(SignatureStep::Reverse)
    } else if body.contains("splice") {
        Ok(SignatureStep::Drop(argument))
    } else if body.contains('%') {
        Ok(SignatureStep::Swap(argument))
    } else {
        Err(SignatureProgramError::UnknownTransformation(name.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// States one player script shaped like the one Youtube serves.
    const PLAYER: &str = r#"
var _yt_player={};
(function(g){var window=this;var Kx={
xW:function(a){a.reverse()},
Jd:function(a,b){a.splice(0,b)},
Ll:function(a,b){var c=a[0];a[0]=a[b%a.length];a[b%a.length]=c}};
g.tj=function(a){a=a.split("");Kx.Ll(a,32);Kx.xW(a,58);Kx.Jd(a,2);Kx.Ll(a,7);
return a.join("")};
})(_yt_player);
"#;

    #[test]
    fn the_recovered_program_is_the_sequence_the_player_states() {
        let program = SignatureProgram::recover(PLAYER).expect("decipher program");

        assert_eq!(
            program.steps(),
            [SignatureStep::Swap(32), SignatureStep::Reverse, SignatureStep::Drop(2), SignatureStep::Swap(7),]
        );
    }

    #[test]
    fn deciphering_applies_every_transformation_in_order() {
        let program = SignatureProgram::recover(PLAYER).expect("decipher program");

        // Applying the same four steps by hand to the same input states the expected answer.
        let mut expected: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
        let position = 32 % expected.len();
        expected.swap(0, position);
        expected.reverse();
        expected.drain(..2);
        let position = 7 % expected.len();
        expected.swap(0, position);

        assert_eq!(program.decipher("abcdefghijklmnopqrstuvwxyz0123456789"), expected.into_iter().collect::<String>());
    }

    #[test]
    fn deciphering_preserves_the_character_count_the_steps_denote() {
        let program = SignatureProgram::recover(PLAYER).expect("decipher program");
        let signature = "abcdefghijklmnopqrstuvwxyz0123456789";

        assert_eq!(program.decipher(signature).chars().count(), signature.len() - 2);
    }

    #[test]
    fn a_script_stating_no_decipher_function_states_no_program() {
        assert_eq!(SignatureProgram::recover("var a = 1;"), Err(SignatureProgramError::MissingFunction));
    }

    #[test]
    fn a_decipher_function_without_its_helper_object_states_no_program() {
        let orphan = r#"g.tj=function(a){a=a.split("");Kx.Ll(a,32);return a.join("")};"#;

        assert_eq!(SignatureProgram::recover(orphan), Err(SignatureProgramError::MissingHelper));
    }

    #[test]
    fn the_empty_program_leaves_a_signature_unchanged() {
        assert_eq!(SignatureProgram::default().decipher("posed"), "posed");
    }
}
