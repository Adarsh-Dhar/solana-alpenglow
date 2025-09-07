#!/bin/bash
# Alpenglow Reproducibility Verification Script
# This script ensures that verification results can be reproduced exactly

set -e

echo "=== Alpenglow Reproducibility Verification ==="
echo "Timestamp: $(date)"
echo "Ensuring verification results are reproducible with fixed seeds"
echo ""

# Create results directory
mkdir -p results/reproducibility_verification
cd results/reproducibility_verification

# Set fixed seeds for deterministic results
export RUST_SEED=12345
export RANDOM_SEED=67890

# Test reproducibility of safety verification
echo "Testing safety verification reproducibility..."

# Run safety verification with fixed seed
cargo run --release --bin safety_verification -- --validators 3 --slots 2 --seed 12345 > safety_run1.log 2>&1

# Run again with same seed
cargo run --release --bin safety_verification -- --validators 3 --slots 2 --seed 12345 > safety_run2.log 2>&1

# Compare results
if diff safety_run1.log safety_run2.log > /dev/null; then
    echo "✅ Safety verification is reproducible"
    echo "safety,3,2,REPRODUCIBLE" >> reproducibility_results.csv
else
    echo "❌ Safety verification is not reproducible"
    echo "safety,3,2,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    echo "Differences found:"
    diff safety_run1.log safety_run2.log
fi

# Test reproducibility of liveness verification
echo "Testing liveness verification reproducibility..."

# Run liveness verification with fixed seed
cargo run --release --bin liveness_verification -- --responsive-stake 80 --seed 12345 > liveness_run1.log 2>&1

# Run again with same seed
cargo run --release --bin liveness_verification -- --responsive-stake 80 --seed 12345 > liveness_run2.log 2>&1

# Compare results
if diff liveness_run1.log liveness_run2.log > /dev/null; then
    echo "✅ Liveness verification is reproducible"
    echo "liveness,80,REPRODUCIBLE" >> reproducibility_results.csv
else
    echo "❌ Liveness verification is not reproducible"
    echo "liveness,80,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    echo "Differences found:"
    diff liveness_run1.log liveness_run2.log
fi

# Test reproducibility of resilience verification
echo "Testing resilience verification reproducibility..."

# Run resilience verification with fixed seed
cargo run --release --bin resilience_verification -- --byzantine-stake 20 --seed 12345 > resilience_run1.log 2>&1

# Run again with same seed
cargo run --release --bin resilience_verification -- --byzantine-stake 20 --seed 12345 > resilience_run2.log 2>&1

# Compare results
if diff resilience_run1.log resilience_run2.log > /dev/null; then
    echo "✅ Resilience verification is reproducible"
    echo "resilience,20,REPRODUCIBLE" >> reproducibility_results.csv
else
    echo "❌ Resilience verification is not reproducible"
    echo "resilience,20,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    echo "Differences found:"
    diff resilience_run1.log resilience_run2.log
fi

# Test reproducibility of certificate verification
echo "Testing certificate verification reproducibility..."

# Run certificate verification with fixed seed
cargo run --release --bin certificate_verification -- --adversary-stake 20 --seed 12345 > certificate_run1.log 2>&1

# Run again with same seed
cargo run --release --bin certificate_verification -- --adversary-stake 20 --seed 12345 > certificate_run2.log 2>&1

# Compare results
if diff certificate_run1.log certificate_run2.log > /dev/null; then
    echo "✅ Certificate verification is reproducible"
    echo "certificate,20,REPRODUCIBLE" >> reproducibility_results.csv
else
    echo "❌ Certificate verification is not reproducible"
    echo "certificate,20,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    echo "Differences found:"
    diff certificate_run1.log certificate_run2.log
fi

# Test reproducibility of dual-path verification
echo "Testing dual-path verification reproducibility..."

# Run dual-path verification with fixed seed
cargo run --release --bin dual_path_test -- --path fast --threshold 80 --stake-percent 90 --seed 12345 > dual_path_run1.log 2>&1

# Run again with same seed
cargo run --release --bin dual_path_test -- --path fast --threshold 80 --stake-percent 90 --seed 12345 > dual_path_run2.log 2>&1

# Compare results
if diff dual_path_run1.log dual_path_run2.log > /dev/null; then
    echo "✅ Dual-path verification is reproducible"
    echo "dual_path,fast,90,REPRODUCIBLE" >> reproducibility_results.csv
else
    echo "❌ Dual-path verification is not reproducible"
    echo "dual_path,fast,90,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    echo "Differences found:"
    diff dual_path_run1.log dual_path_run2.log
fi

# Test reproducibility across different configurations
echo "Testing reproducibility across different configurations..."

configurations=(
    "2,1"
    "3,2"
    "4,3"
    "5,2"
)

for config in "${configurations[@]}"; do
    IFS=',' read -r validators slots <<< "$config"
    echo "Testing configuration: $validators validators, $slots slots"
    
    # Run with fixed seed
    cargo run --release --bin safety_verification -- --validators $validators --slots $slots --seed 12345 > config_${validators}v_${slots}s_run1.log 2>&1
    
    # Run again with same seed
    cargo run --release --bin safety_verification -- --validators $validators --slots $slots --seed 12345 > config_${validators}v_${slots}s_run2.log 2>&1
    
    # Compare results
    if diff config_${validators}v_${slots}s_run1.log config_${validators}v_${slots}s_run2.log > /dev/null; then
        echo "✅ Configuration $validators,$slots is reproducible"
        echo "config,$validators,$slots,REPRODUCIBLE" >> reproducibility_results.csv
    else
        echo "❌ Configuration $validators,$slots is not reproducible"
        echo "config,$validators,$slots,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    fi
done

# Test reproducibility with different random number generators
echo "Testing reproducibility with different RNGs..."

# Test with different RNG seeds
for rng_seed in 11111 22222 33333 44444 55555; do
    echo "Testing with RNG seed: $rng_seed"
    
    # Run with RNG seed
    RUST_SEED=$rng_seed cargo run --release --bin safety_verification -- --validators 3 --slots 2 --seed $rng_seed > rng_${rng_seed}_run1.log 2>&1
    
    # Run again with same RNG seed
    RUST_SEED=$rng_seed cargo run --release --bin safety_verification -- --validators 3 --slots 2 --seed $rng_seed > rng_${rng_seed}_run2.log 2>&1
    
    # Compare results
    if diff rng_${rng_seed}_run1.log rng_${rng_seed}_run2.log > /dev/null; then
        echo "✅ RNG seed $rng_seed is reproducible"
        echo "rng,$rng_seed,REPRODUCIBLE" >> reproducibility_results.csv
    else
        echo "❌ RNG seed $rng_seed is not reproducible"
        echo "rng,$rng_seed,NOT_REPRODUCIBLE" >> reproducibility_results.csv
    fi
done

# Generate reproducibility summary
echo ""
echo "=== Reproducibility Verification Summary ==="

# Count reproducible tests
total_tests=$(wc -l < reproducibility_results.csv)
reproducible_tests=$(grep -c "REPRODUCIBLE" reproducibility_results.csv || true)
not_reproducible_tests=$(grep -c "NOT_REPRODUCIBLE" reproducibility_results.csv || true)

echo "Total tests: $total_tests"
echo "Reproducible: $reproducible_tests"
echo "Not reproducible: $not_reproducible_tests"

if [ $not_reproducible_tests -eq 0 ]; then
    echo "🎉 ALL TESTS ARE REPRODUCIBLE!"
    echo "Verification results can be reproduced exactly with fixed seeds."
else
    echo "⚠️  Some tests are not reproducible."
    echo "Check the detailed results for non-reproducible tests."
fi

# Generate detailed report
echo ""
echo "=== Detailed Reproducibility Report ==="

# Check each test type
for test_type in safety liveness resilience certificate dual_path; do
    count=$(grep "^$test_type," reproducibility_results.csv | wc -l)
    reproducible=$(grep "^$test_type," reproducibility_results.csv | grep -c "REPRODUCIBLE" || true)
    
    if [ $count -gt 0 ]; then
        reproducibility_rate=$((reproducible * 100 / count))
        echo "$test_type: $reproducibility_rate% reproducible ($reproducible/$count)"
    fi
done

# Check configuration reproducibility
echo ""
echo "Configuration Reproducibility:"
config_reproducible=$(grep "^config," reproducibility_results.csv | grep -c "REPRODUCIBLE" || true)
config_total=$(grep "^config," reproducibility_results.csv | wc -l)

if [ $config_total -gt 0 ]; then
    config_rate=$((config_reproducible * 100 / config_total))
    echo "Configurations: $config_rate% reproducible ($config_reproducible/$config_total)"
fi

# Check RNG reproducibility
echo ""
echo "RNG Reproducibility:"
rng_reproducible=$(grep "^rng," reproducibility_results.csv | grep -c "REPRODUCIBLE" || true)
rng_total=$(grep "^rng," reproducibility_results.csv | wc -l)

if [ $rng_total -gt 0 ]; then
    rng_rate=$((rng_reproducible * 100 / rng_total))
    echo "RNG seeds: $rng_rate% reproducible ($rng_reproducible/$rng_total)"
fi

# Generate recommendations
echo ""
echo "=== Recommendations ==="

if [ $not_reproducible_tests -eq 0 ]; then
    echo "✅ All tests are reproducible. The verification suite is ready for production use."
    echo "✅ Results can be trusted and reproduced by other researchers."
    echo "✅ Fixed seeds ensure deterministic behavior across different environments."
else
    echo "⚠️  Some tests are not reproducible. Consider the following actions:"
    echo "1. Review non-reproducible tests and identify sources of non-determinism"
    echo "2. Ensure all random number generators use fixed seeds"
    echo "3. Check for time-dependent behavior or external dependencies"
    echo "4. Verify that all verification binaries accept seed parameters"
fi

echo ""
echo "Reproducibility verification completed. Check reproducibility_results.csv for detailed results."
