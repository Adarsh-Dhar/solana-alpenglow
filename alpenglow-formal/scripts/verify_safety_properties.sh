#!/bin/bash
# Alpenglow Safety Property Verification Script
# This script verifies the core safety properties of the Alpenglow consensus protocol

set -e

echo "=== Alpenglow Safety Property Verification ==="
echo "Timestamp: $(date)"
echo "Configuration: Testing various validator and slot combinations"
echo ""

# Create results directory
mkdir -p results/safety_verification
cd results/safety_verification

# Test different configurations
echo "Running safety verification tests..."

# Test 1: Basic safety with 2 validators, 1 slot
echo "Test 1: 2 validators, 1 slot"
cargo run --release --bin safety_verification -- --validators 2 --slots 1 --seed 12345 > test_2v_1s.log 2>&1

if grep -q "Property 'safety' is always true" test_2v_1s.log; then
    echo "✅ SAFETY VERIFIED: 2 validators, 1 slot"
    echo "2,1,SUCCESS" >> safety_results.csv
else
    echo "❌ SAFETY VIOLATION: 2 validators, 1 slot"
    echo "2,1,FAILURE" >> safety_results.csv
fi

# Test 2: 3 validators, 2 slots
echo "Test 2: 3 validators, 2 slots"
cargo run --release --bin safety_verification -- --validators 3 --slots 2 --seed 12345 > test_3v_2s.log 2>&1

if grep -q "Property 'safety' is always true" test_3v_2s.log; then
    echo "✅ SAFETY VERIFIED: 3 validators, 2 slots"
    echo "3,2,SUCCESS" >> safety_results.csv
else
    echo "❌ SAFETY VIOLATION: 3 validators, 2 slots"
    echo "3,2,FAILURE" >> safety_results.csv
fi

# Test 3: 4 validators, 3 slots
echo "Test 3: 4 validators, 3 slots"
cargo run --release --bin safety_verification -- --validators 4 --slots 3 --seed 12345 > test_4v_3s.log 2>&1

if grep -q "Property 'safety' is always true" test_4v_3s.log; then
    echo "✅ SAFETY VERIFIED: 4 validators, 3 slots"
    echo "4,3,SUCCESS" >> safety_results.csv
else
    echo "❌ SAFETY VIOLATION: 4 validators, 3 slots"
    echo "4,3,FAILURE" >> safety_results.csv
fi

# Test 4: 5 validators, 4 slots
echo "Test 4: 5 validators, 4 slots"
cargo run --release --bin safety_verification -- --validators 5 --slots 4 --seed 12345 > test_5v_4s.log 2>&1

if grep -q "Property 'safety' is always true" test_5v_4s.log; then
    echo "✅ SAFETY VERIFIED: 5 validators, 4 slots"
    echo "5,4,SUCCESS" >> safety_results.csv
else
    echo "❌ SAFETY VIOLATION: 5 validators, 4 slots"
    echo "5,4,FAILURE" >> safety_results.csv
fi

# Generate summary report
echo ""
echo "=== Safety Verification Summary ==="
echo "Validators,Slots,Result" > safety_summary.csv
cat safety_results.csv >> safety_summary.csv

total_tests=$(wc -l < safety_results.csv)
successful_tests=$(grep -c "SUCCESS" safety_results.csv || true)
failed_tests=$(grep -c "FAILURE" safety_results.csv || true)

echo "Total tests: $total_tests"
echo "Successful: $successful_tests"
echo "Failed: $failed_tests"

if [ "$failed_tests" -eq 0 ]; then
    echo "🎉 ALL SAFETY TESTS PASSED!"
    exit 0
else
    echo "⚠️  Some safety tests failed. Check logs for details."
    exit 1
fi
