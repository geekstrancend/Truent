//! EVM generator implementation.

use truent_core::model::{GenerationOutput, Invariant, ProgramModel};
use truent_core::traits::CodeGenerator;
use truent_core::Result;
use tracing::info;

/// Code generator for EVM (Solidity) contracts.
pub struct EvmGenerator;

impl CodeGenerator for EvmGenerator {
    fn generate(
        &self,
        program: &ProgramModel,
        invariants: &[Invariant],
    ) -> Result<GenerationOutput> {
        info!(
            "Generating Solidity modifiers for {} with {} invariants",
            program.name,
            invariants.len()
        );

        let mut assertions = Vec::new();
        for inv in invariants {
            assertions.push(format!(
                "require({}, \"Invariant: {}\");",
                inv.expression, inv.name
            ));
        }

        let code = format!(
            "// Generated Solidity invariant checks for {}\n// {} checks\n",
            program.name,
            assertions.len()
        );

        Ok(GenerationOutput {
            code,
            assertions,
            tests: None,
            coverage_percent: 0,
        })
    }

    fn chain(&self) -> &str {
        "evm"
    }
}
