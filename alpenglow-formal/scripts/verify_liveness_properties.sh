#!/bin/bash
# Alpenglow Liveness Property Verification Script
# This script verifies liveness properties under various network conditions

set -e

echo "=== Alpenglow Liveness Property Verification ==="
echo "Timestamp: $(date)"
echo "Testing liveness under different responsive stake percentages"
echo ""

# Create results directory
mkdir -p results/liveness_verification
cd results/liveness_verification

# Initialize results file
echo "Stake_Percent,Simulation_Run,Result,Finalization_Time" > liveness_results.csv

# Test different responsive stake percentages
for stake_pct in 50 60 70 80 90; do
    echo "Testing with $stake_pct% responsive stake..."
    
    success_count=0
    total_runs=100
    
    for i in $(seq 1 $total_runs); do
        # Run liveness simulation
        cargo run --release --bin liveness_verification -- --responsive-stake $stake_pct --seed $i > run_${stake_pct}pct_${i}.log 2>&1
        
        # Check if liveness succeeded
        if grep -q "Liveness Success" run_${stake_pct}pct_${i}.log; then
            finalization_time=$(grep "Liveness Success" run_${stake_pct}pct_${i}.log | grep -o "T=[0-9]*" | cut -d= -f2)
            echo "$stake_pct,$i,SUCCESS,$finalization_time" >> liveness_results.csv
            ((success_count++))
        else
            echo "$stake_pct,$i,FAILURE,-1" >> liveness_results.csv
        fi
    done
    
    success_rate=$((success_count * 100 / total_runs))
    echo "Success rate with $stake_pct% stake: $success_rate% ($success_count/$total_runs)"
done

# Test fast path vs slow path performance
echo ""
echo "Testing Fast Path vs Slow Path performance..."

# Fast path test (90% responsive stake)
echo "Fast Path Test (90% responsive stake):"
fast_path_success=0
for i in $(seq 1 50); do
    cargo run --release --bin liveness_verification -- --responsive-stake 90 --seed $i > fast_path_${i}.log 2>&1
    if grep -q "Liveness Success" fast_path_${i}.log; then
        ((fast_path_success++))
    fi
done
fast_path_rate=$((fast_path_success * 100 / 50))
echo "Fast Path Success Rate: $fast_path_rate%"

# Slow path test (70% responsive stake)
echo "Slow Path Test (70% responsive stake):"
slow_path_success=0
for i in $(seq 1 50); do
    cargo run --release --bin liveness_verification -- --responsive-stake 70 --seed $i > slow_path_${i}.log 2>&1
    if grep -q "Liveness Success" slow_path_${i}.log; then
        ((slow_path_success++))
    fi
done
slow_path_rate=$((slow_path_success * 100 / 50))
echo "Slow Path Success Rate: $slow_path_rate%"

# Generate summary report
echo ""
echo "=== Liveness Verification Summary ==="
echo "Stake_Percent,Success_Rate,Total_Runs" > liveness_summary.csv

for stake_pct in 50 60 70 80 90; do
    success_count=$(grep "^$stake_pct," liveness_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / total_runs))
    echo "$stake_pct,$success_rate,$total_runs" >> liveness_summary.csv
done

echo "Liveness verification completed. Check liveness_summary.csv for detailed results."

# Verify critical thresholds
if [ $fast_path_rate -ge 90 ]; then
    echo "✅ Fast Path (90% stake) meets performance requirements"
else
    echo "❌ Fast Path (90% stake) below performance requirements"
fi

if [ $slow_path_rate -ge 80 ]; then
    echo "✅ Slow Path (70% stake) meets performance requirements"
else
    echo "❌ Slow Path (70% stake) below performance requirements"
fi
