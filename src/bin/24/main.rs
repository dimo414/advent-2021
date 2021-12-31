use anyhow::{anyhow, bail, Context, ensure, Error, Result};

use std::collections::{HashMap, VecDeque};
use std::str::FromStr;
use advent_2021::console::interactive;

fn main() -> Result<()> {
    let program: Program = include_str!("input.txt").parse()?;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        args[0].parse::<u64>().with_context(|| format!("Invalid input {:?}", &args[0]))?;
        let mut digits: VecDeque<_> = args[0].chars().map(|d| d.to_digit(10).expect("Impossible")).collect();
        let parts = program.split_at_reads();
        let mut alu = LogicUnit::new();
        alu.execute(&parts[0], &[])?;

        for part in &parts[1..] {
            let d = digits.pop_front().expect("Present") as i64;
            alu.execute(part, &[d])?;
            println!("{:?}", alu);
        }
    } else {
        let found = explore_program(&program)?;
        let max = found.values().map(|(_,x)| x).max().expect("No max found");
        let min = found.values().map(|(n,_)| n).min().expect("No min found");
        println!("Max: {}\nMin: {}", max, min);
    }

    Ok(())
}

#[derive(Debug, Copy, Clone)]
enum Argument {
    Register(char),
    Literal(i64),
}

impl FromStr for Argument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "w"|"x"|"y"|"z" => Argument::Register(s.chars().next().expect("Must be present")),
            _ => Argument::Literal(s.parse()?),
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Inp(Argument),
    Add(Argument, Argument),
    Mul(Argument, Argument),
    Div(Argument, Argument),
    Mod(Argument, Argument),
    Eql(Argument, Argument),
}

impl FromStr for Instruction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(' ').collect();
        ensure!(parts.len() == 3 || (parts.len() == 2 && parts[0] == "inp"));
        Ok(match parts[0] {
            "inp" => Instruction::Inp(parts[1].parse()?),
            "add" => Instruction::Add(parts[1].parse()?, parts[2].parse()?),
            "mul" => Instruction::Mul(parts[1].parse()?, parts[2].parse()?),
            "div" => Instruction::Div(parts[1].parse()?, parts[2].parse()?),
            "mod" => Instruction::Mod(parts[1].parse()?, parts[2].parse()?),
            "eql" => Instruction::Eql(parts[1].parse()?, parts[2].parse()?),
            _ => bail!("Invalid instruction: {:?}", s),
        })
    }
}

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    // Returns one or more programs, where index 0 is a constant prelude, and each subsequent index
    // begins with an INP command and contains all subsequent commands until the next INP.
    fn split_at_reads(&self) -> Vec<Program> {
        let mut ret = vec![Vec::new()];

        for instr in &self.instructions {
            if let Instruction::Inp(_) = instr {
                ret.push(Vec::new());
            }
            ret.last_mut().expect("Non-empty").push(*instr);
        }

        ret.into_iter().map(|instructions| Program{ instructions }).collect()
    }
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let instructions = s.lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
            .map(|l| l.parse())
            .collect::<Result<Vec<_>>>()?;
        Ok(Program { instructions })
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
struct LogicUnit {
    registers: [i64; 4],
}

impl LogicUnit {
    fn new() -> LogicUnit {
        LogicUnit{ registers: [0; 4], }
    }

    fn register_idx(variable: char) -> Result<usize> {
        ensure!(('w'..='z').contains(&variable));
        Ok(variable as usize - 'w' as usize)
    }

    fn read(&self, arg: &Argument) -> Result<i64> {
        Ok(match arg {
            Argument::Register(c) => self.registers[LogicUnit::register_idx(*c)?],
            Argument::Literal(n) => *n,
        })
    }

    fn write(&mut self, arg: &Argument, value: i64) -> Result<()> {
        if let Argument::Register(c) = arg {
            self.registers[LogicUnit::register_idx(*c)?] = value;
            return Ok(());
        }
        bail!("Cannot write to {:?}", arg)
    }

    fn execute(&mut self, program: &Program, input: &[i64]) -> Result<()> {
        let mut input_idx = 0;
        for instr in &program.instructions {
            match instr {
                Instruction::Inp(a) => {
                    self.write(a, *input.get(input_idx).ok_or_else(|| anyhow!("Insufficient input"))?)?;
                    input_idx+=1;
                },
                Instruction::Add(a, b) => {
                    self.write(a, self.read(a)? + self.read(b)?)?;
                },
                Instruction::Mul(a, b) => {
                    self.write(a, self.read(a)? * self.read(b)?)?;
                },
                Instruction::Div(a, b) => {
                    self.write(a, self.read(a)? / self.read(b)?)?;
                },
                Instruction::Mod(a, b) => {
                    self.write(a, self.read(a)? % self.read(b)?)?;
                },
                Instruction::Eql(a, b) => {
                    let eq = self.read(a)? == self.read(b)?;
                    self.write(a, if eq { 1 } else { 0 })?;
                },
            }
        }
        ensure!(input_idx == input.len(), "Not all input consumed.");
        Ok(())
    }
}

fn explore_program(program: &Program) -> Result<HashMap<LogicUnit, (u64, u64)>> {
    let parts = program.split_at_reads();
    let mut alu = LogicUnit::new();
    alu.execute(&parts[0], &[])?;
    let mut unique_states = HashMap::new();
    unique_states.insert(alu, (0u64, 0u64));
    for part in &parts[1..] {
        let mut next_states = HashMap::new();
        for (alu, (min, max)) in &unique_states {
            for d in 1..=9 {
                let mut low = min * 10 + d;
                let mut high = max * 10 + d;
                let mut alu = *alu;
                alu.execute(part, &[d as i64])?;

                // heuristically discard zs that get too large
                // https://old.reddit.com/r/adventofcode/comments/rnj7r7/2021_day_24_how_do_you_approach_this/hpxov96/
                if alu.registers[3] > 1_000_000 { continue; }

                // Track the min and max prefixes found that reach this state; discard other inputs
                if let Some(&(min, max)) = next_states.get(&alu) {
                    low = std::cmp::min(low, min);
                    high = std::cmp::max(high, max);
                }
                next_states.insert(alu, (low, high));
            }
        }
        unique_states = next_states;
        if interactive!() {
            println!("Unique States: {}", unique_states.len());
        }
    }
    Ok(unique_states.into_iter().filter(|(lu, _)| lu.registers[3] == 0).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negate() {
        let program: Program = "inp x\nmul x -1".parse().unwrap();
        let mut alu = LogicUnit::new();
        alu.execute(&program, &[10]).unwrap();
        assert_eq!(alu.registers, [0, -10, 0, 0]);
    }

    #[test]
    fn three_times_larger() {
        let program: Program = "inp z\ninp x\nmul z 3\neql z x".parse().unwrap();
        let mut alu = LogicUnit::new();
        alu.execute(&program, &[10, 20]).unwrap();
        assert_eq!(alu.registers, [0, 20, 0, 0]); // z=0

        let mut alu = LogicUnit::new();
        alu.execute(&program, &[10, 30]).unwrap();
        assert_eq!(alu.registers, [0, 30, 0, 1]); // z=1
    }

    #[test]
    fn to_binary() {
        let program: Program = "inp w\nadd z w\nmod z 2\ndiv w 2\nadd y w\nmod y 2\ndiv w 2\nadd x w\nmod x 2\ndiv w 2\nmod w 2".parse().unwrap();
        let mut alu = LogicUnit::new();
        alu.execute(&program, &[10]).unwrap();
        assert_eq!(alu.registers, [1, 0, 1, 0]);
    }

    #[test]
    fn input_test() {
        let program: Program = include_str!("alt-input.txt").parse().unwrap();
        let found = explore_program(&program).unwrap();
        let max = *found.values().map(|(_,x)| x).max().expect("No max found");
        let min = *found.values().map(|(n,_)| n).min().expect("No min found");
        assert_eq!(max, 59996912981939);
        assert_eq!(min, 17241911811915);
    }
}