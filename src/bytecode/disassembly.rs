//! A unit written out as text, one instruction a line, under the source
//! line each group of instructions was compiled from.

use std::fmt::Write;

use crate::bytecode::instruction::{Instruction, Operand};
use crate::bytecode::unit::{StartKind, Unit};

/// `unit` as text. `file` names where the unit was written and `source` is
/// that file's text, which the line each instruction came from is read
/// from.
pub fn disassemble(unit: &Unit, file: &str, source: Option<&str>) -> String {
    let lines: Vec<&str> = source
        .map(|text| text.lines().collect())
        .unwrap_or_default();
    let mut text = format!("== {file}\n");
    let mut line_written = None;
    for (at, instruction) in unit.instructions.iter().enumerate() {
        let line = unit.positions[at].line;
        // A body with no statements has no line of its own.
        if line > 0 && line_written != Some(line) {
            line_written = Some(line);
            match line.checked_sub(1).and_then(|index| lines.get(index)) {
                Some(source_line) => {
                    let _ = writeln!(text, "{line:>5} | {}", source_line.trim_end());
                }
                None => {
                    let _ = writeln!(text, "{line:>5} |");
                }
            }
        }
        let _ = write!(text, "        {at:04}  {instruction:?}");
        if let Some(note) = note(unit, instruction) {
            let _ = write!(text, "  ; {note}");
        }
        text.push('\n');
    }
    text
}

/// What an instruction's side table operand names.
fn note(unit: &Unit, instruction: &Instruction) -> Option<String> {
    let name = |index: u16| {
        unit.names
            .get(usize::from(index))
            .map(|name| name.text.clone())
    };
    match *instruction {
        Instruction::LoadConstant { constant, .. } => unit
            .constants
            .get(usize::from(constant))
            .map(|value| value.to_string()),
        Instruction::LoadSymbol { name: index, .. } => name(index).map(|text| format!(":{text}")),
        Instruction::NewString { text, .. } => name(text).map(|text| format!("{text:?}")),
        Instruction::GetLocal { slot, .. }
        | Instruction::SetLocal { slot, .. }
        | Instruction::Unhoist { slot }
        | Instruction::DeclareLocal { slot } => unit
            .locals
            .get(usize::from(slot))
            .map(|local| local.text.clone()),
        Instruction::LoadRegex { pattern, .. } => unit
            .regexes
            .get(usize::from(pattern))
            .map(|regex| format!("/{}/{}", regex.pattern, regex.flags)),
        Instruction::GetTopLevelConstant { name: index, .. } => {
            name(index).map(|text| format!("::{text}"))
        }
        Instruction::GetIdentifier { name: index, .. }
        | Instruction::GetScopedConstant { name: index, .. }
        | Instruction::GetClassVariable { name: index, .. }
        | Instruction::SetClassVariable { name: index, .. }
        | Instruction::GetInstanceVariable { name: index, .. }
        | Instruction::SetInstanceVariable { name: index, .. }
        | Instruction::GetConstant { name: index, .. }
        | Instruction::SetConstant { name: index, .. } => name(index),
        Instruction::GetGlobal { name: index, .. } | Instruction::SetGlobal { name: index, .. } => {
            name(index).map(|text| format!("${text}"))
        }
        Instruction::Call { site, .. } => unit
            .call_sites
            .get(usize::from(site))
            .map(|site| format!("{}/{}", site.name, site.argument_count)),
        Instruction::Add { left, right, .. }
        | Instruction::Subtract { left, right, .. }
        | Instruction::Multiply { left, right, .. }
        | Instruction::Divide { left, right, .. }
        | Instruction::Modulo { left, right, .. }
        | Instruction::Equal { left, right, .. }
        | Instruction::NotEqual { left, right, .. }
        | Instruction::Less { left, right, .. }
        | Instruction::LessEqual { left, right, .. }
        | Instruction::Greater { left, right, .. }
        | Instruction::GreaterEqual { left, right, .. } => {
            let constants: Vec<String> = [left, right]
                .into_iter()
                .filter_map(|operand| match Operand::read(operand, 0) {
                    Operand::Constant(at) => unit.constants.get(at).map(|value| value.to_string()),
                    Operand::Register(_) => None,
                })
                .collect();
            (!constants.is_empty()).then(|| constants.join(", "))
        }
        _ => None,
    }
}

/// One instruction of a unit as `RubyVM::InstructionSequence` lists it.
pub struct Listed {
    pub name: &'static str,
    /// Each operand's field name and value, in the order the instruction
    /// declares them.
    pub operands: Vec<(String, i64)>,
    pub line: usize,
    /// Whether the instruction starts a statement that is a line event.
    pub line_event: bool,
    /// What a side table operand names.
    pub note: Option<String>,
}

/// The instructions of `unit`, in order.
pub fn listing(unit: &Unit) -> Vec<Listed> {
    unit.instructions
        .iter()
        .enumerate()
        .map(|(at, instruction)| Listed {
            name: instruction.name(),
            operands: operands(instruction),
            line: unit.positions[at].line,
            line_event: unit
                .statement_starts
                .get(at)
                .copied()
                .flatten()
                .is_some_and(|start| match start.kind {
                    StartKind::Statement { line_event } | StartKind::Terminal { line_event } => {
                        line_event
                    }
                    StartKind::BodyStatement { runs } => runs,
                    StartKind::InBranch { .. } => false,
                }),
            note: note(unit, instruction),
        })
        .collect()
}

/// The operands of `instruction`, read from its derived `Debug` text, which
/// is `Name { field: value, ... }` with every value an integer.
fn operands(instruction: &Instruction) -> Vec<(String, i64)> {
    let written = format!("{instruction:?}");
    let Some((_, fields)) = written.split_once(" { ") else {
        return Vec::new();
    };
    fields
        .trim_end_matches(" }")
        .split(", ")
        .filter_map(|field| {
            let (name, value) = field.split_once(": ")?;
            Some((name.to_string(), value.parse().ok()?))
        })
        .collect()
}
