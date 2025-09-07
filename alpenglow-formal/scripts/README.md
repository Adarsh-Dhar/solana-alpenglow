# Alpenglow Formal Verification Scripts

This directory contains proof scripts for the Alpenglow consensus protocol formal verification. These scripts generate reproducible verification results and comprehensive reports.

## Script Overview

### Core Verification Scripts

1. **`verify_safety_properties.sh`** - Verifies core safety properties
   - Tests 2-5 validators with 1-4 slots
   - Ensures no conflicting blocks can be finalized
   - Generates safety verification report

2. **`verify_liveness_properties.sh`** - Verifies liveness properties
   - Tests 50-90% responsive stake scenarios
   - Verifies progress guarantee with >60% honest participation
   - Tests fast path (80% threshold) vs slow path (60% threshold)

3. **`verify_resilience_properties.sh`** - Verifies resilience against attacks
   - Tests Byzantine attacks with 10-30% adversary stake
   - Tests liveness with 10-30% non-responsive stake
   - Tests network partition recovery

4. **`verify_certificate_uniqueness.sh`** - Verifies certificate properties
   - Tests certificate uniqueness under adversarial conditions
   - Tests equivocation, vote splitting, and nothing-at-stake attacks
   - Ensures no conflicting certificates can be formed

5. **`verify_leader_rotation.sh`** - Verifies leader management
   - Tests leader rotation and window management
   - Tests BadWindow flag management
   - Tests failure handling and recovery

6. **`verify_timeout_handling.sh`** - Verifies timeout mechanisms
   - Tests timeout handling with various delays
   - Tests skip certificate generation
   - Tests BadWindow flag triggering

7. **`verify_rotor_sampling.sh`** - Verifies rotor sampling
   - Tests message dissemination efficiency
   - Tests stake-weighted selection
   - Tests fault tolerance and scalability

8. **`verify_dual_path_finality.sh`** - Verifies dual-path finality
   - Tests fast path (80% threshold) finalization
   - Tests slow path (60% threshold) finalization
   - Verifies bounded finalization time

9. **`verify_bounded_finalization.sh`** - Verifies bounded finalization time
   - Tests min(δ₈₀%, 2δ₆₀%) bound
   - Tests network delay impact
   - Tests concurrent and partial network finalization

### Utility Scripts

10. **`benchmark_verification.sh`** - Performance benchmarking
    - Tests verification performance across configurations
    - Measures memory usage and execution time
    - Analyzes scalability characteristics

11. **`ensure_reproducibility.sh`** - Reproducibility verification
    - Ensures results can be reproduced with fixed seeds
    - Tests deterministic behavior across runs
    - Validates verification consistency

12. **`run_all_verifications.sh`** - Complete verification suite
    - Runs all verification scripts
    - Generates comprehensive reports
    - Provides overall verification status

13. **`generate_summary_report.sh`** - Report generation
    - Generates human-readable summary reports
    - Creates machine-readable JSON results
    - Provides detailed analysis and recommendations

## Usage

### Running Individual Scripts

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Run specific verification
./scripts/verify_safety_properties.sh
./scripts/verify_liveness_properties.sh
./scripts/verify_resilience_properties.sh
```

### Running Complete Verification Suite

```bash
# Run all verifications
./scripts/run_all_verifications.sh

# Generate summary report
./scripts/generate_summary_report.sh
```

### Running Performance Benchmarks

```bash
# Run performance benchmarks
./scripts/benchmark_verification.sh

# Ensure reproducibility
./scripts/ensure_reproducibility.sh
```

## Output Structure

Each script generates results in the following structure:

```
results/
├── safety_verification/
│   ├── safety_results.csv
│   └── safety_summary.csv
├── liveness_verification/
│   ├── liveness_results.csv
│   └── liveness_summary.csv
├── resilience_verification/
│   └── resilience_results.csv
├── certificate_verification/
│   └── certificate_results.csv
├── leader_verification/
│   └── leader_results.csv
├── timeout_verification/
│   └── timeout_results.csv
├── rotor_verification/
│   └── rotor_results.csv
├── dual_path_verification/
│   └── dual_path_results.csv
├── bounded_finalization_verification/
│   └── bounded_finalization_results.csv
└── benchmark_verification/
    ├── benchmark_results.csv
    ├── state_space_analysis.csv
    └── memory_analysis.csv
```

## Report Generation

The scripts generate comprehensive reports:

- **`verification_summary.md`** - Human-readable summary
- **`verification_results.json`** - Machine-readable results
- **`verification_summary.csv`** - Tabular summary

## Configuration

Scripts can be configured by modifying parameters:

- **Validator counts**: 2-10 validators
- **Slot counts**: 1-5 slots
- **Stake percentages**: 50-100%
- **Network delays**: 10-50ms
- **Test iterations**: 30-100 runs per test

## Dependencies

- Rust and Cargo
- Bash shell
- Standard Unix utilities (awk, grep, sort, etc.)
- bc (for mathematical calculations)

## Reproducibility

All scripts use fixed seeds to ensure reproducible results:

- **RUST_SEED=12345**
- **RANDOM_SEED=67890**
- **Test-specific seeds**: 1-1000

## Verification Properties

The scripts verify the following critical properties:

### Safety Properties
- No two conflicting blocks can be finalized in the same slot
- Chain consistency under up to 20% Byzantine stake
- Certificate uniqueness and non-equivocation

### Liveness Properties
- Progress guarantee under partial synchrony with >60% honest participation
- Fast path completion in one round with >80% responsive stake
- Bounded finalization time min(δ₈₀%, 2δ₆₀%)

### Resilience Properties
- Safety maintained with ≤20% Byzantine stake
- Liveness maintained with ≤20% non-responsive stake
- Network partition recovery guarantees

### Performance Properties
- Scalability to 100+ validators
- Memory efficiency
- Execution time bounds

## Troubleshooting

### Common Issues

1. **Script permissions**: Ensure scripts are executable
   ```bash
   chmod +x scripts/*.sh
   ```

2. **Missing dependencies**: Install required tools
   ```bash
   # Install bc for mathematical calculations
   sudo apt-get install bc  # Ubuntu/Debian
   brew install bc          # macOS
   ```

3. **Rust compilation errors**: Ensure Rust is properly installed
   ```bash
   rustup update
   cargo build --release
   ```

4. **Memory issues**: Reduce validator counts for large tests
   ```bash
   # Edit script parameters
   --validators 3 --slots 2
   ```

### Getting Help

- Check script output for error messages
- Review generated log files in results directories
- Ensure all required binaries are compiled
- Verify input parameters are valid

## Contributing

When adding new verification scripts:

1. Follow the existing naming convention
2. Include comprehensive error handling
3. Generate structured output (CSV format)
4. Add to the main verification suite
5. Update this README with script description

## License

This verification suite is part of the Alpenglow formal verification project and is licensed under Apache 2.0.
