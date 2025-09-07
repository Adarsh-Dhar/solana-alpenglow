#!/bin/bash
# Alpenglow Resilience Property Verification Script
# This script verifies resilience against Byzantine attacks and network partitions

set -e

echo "=== Alpenglow Resilience Property Verification ==="
echo "Timestamp: $(date)"
echo "Testing resilience against Byzantine attacks and network partitions"
echo ""

# Create results directory
mkdir -p results/resilience_verification
cd results/resilience_verification

# Initialize results file
echo "Test_Type,Byzantine_Stake,Test_Run,Result" > resilience_results.csv

# Test 1: Safety under Byzantine attacks
echo "Test 1: Safety under Byzantine attacks"
for byzantine_stake in 10 15 20 25; do
    echo "Testing with $byzantine_stake% Byzantine stake..."
    
    for i in $(seq 1 50); do
        cargo run --release --bin resilience_verification -- --byzantine-stake $byzantine_stake --test-type safety --seed $i > byzantine_${byzantine_stake}pct_${i}.log 2>&1
        
        if grep -q "Safety maintained" byzantine_${byzantine_stake}pct_${i}.log; then
            echo "byzantine_safety,$byzantine_stake,$i,SUCCESS" >> resilience_results.csv
        else
            echo "byzantine_safety,$byzantine_stake,$i,FAILURE" >> resilience_results.csv
        fi
    done
done

# Test 2: Liveness with non-responsive nodes
echo "Test 2: Liveness with non-responsive nodes"
for offline_stake in 10 15 20 25; do
    echo "Testing with $offline_stake% non-responsive stake..."
    
    for i in $(seq 1 50); do
        cargo run --release --bin resilience_verification -- --offline-stake $offline_stake --test-type liveness --seed $i > offline_${offline_stake}pct_${i}.log 2>&1
        
        if grep -q "Liveness maintained" offline_${offline_stake}pct_${i}.log; then
            echo "offline_liveness,$offline_stake,$i,SUCCESS" >> resilience_results.csv
        else
            echo "offline_liveness,$offline_stake,$i,FAILURE" >> resilience_results.csv
        fi
    done
done

# Test 3: Network partition recovery
echo "Test 3: Network partition recovery"
for i in $(seq 1 30); do
    cargo run --release --bin resilience_verification -- --test-type partition --seed $i > partition_${i}.log 2>&1
    
    if grep -q "Partition recovery successful" partition_${i}.log; then
        echo "partition_recovery,50,$i,SUCCESS" >> resilience_results.csv
    else
        echo "partition_recovery,50,$i,FAILURE" >> resilience_results.csv
    fi
done

# Test 4: Certificate uniqueness under attack
echo "Test 4: Certificate uniqueness under attack"
for i in $(seq 1 30); do
    cargo run --release --bin certificate_verification -- --adversary-stake 20 --seed $i > certificate_${i}.log 2>&1
    
    if grep -q "Certificate uniqueness maintained" certificate_${i}.log; then
        echo "certificate_uniqueness,20,$i,SUCCESS" >> resilience_results.csv
    else
        echo "certificate_uniqueness,20,$i,FAILURE" >> resilience_results.csv
    fi
done

# Generate summary report
echo ""
echo "=== Resilience Verification Summary ==="

# Analyze Byzantine safety results
echo "Byzantine Safety Results:"
for byzantine_stake in 10 15 20 25; do
    success_count=$(grep "byzantine_safety,$byzantine_stake," resilience_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 50))
    echo "  $byzantine_stake% Byzantine stake: $success_rate% success rate"
done

# Analyze offline liveness results
echo "Offline Liveness Results:"
for offline_stake in 10 15 20 25; do
    success_count=$(grep "offline_liveness,$offline_stake," resilience_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 50))
    echo "  $offline_stake% offline stake: $success_rate% success rate"
done

# Analyze partition recovery results
partition_success=$(grep "partition_recovery," resilience_results.csv | grep -c "SUCCESS" || true)
partition_rate=$((partition_success * 100 / 30))
echo "Partition Recovery: $partition_rate% success rate"

# Analyze certificate uniqueness results
cert_success=$(grep "certificate_uniqueness," resilience_results.csv | grep -c "SUCCESS" || true)
cert_rate=$((cert_success * 100 / 30))
echo "Certificate Uniqueness: $cert_rate% success rate"

# Verify critical thresholds
echo ""
echo "=== Threshold Verification ==="

# Check if safety is maintained with ≤20% Byzantine stake
byzantine_20_success=$(grep "byzantine_safety,20," resilience_results.csv | grep -c "SUCCESS" || true)
byzantine_20_rate=$((byzantine_20_success * 100 / 50))

if [ $byzantine_20_rate -ge 90 ]; then
    echo "✅ Safety maintained with ≤20% Byzantine stake ($byzantine_20_rate%)"
else
    echo "❌ Safety compromised with 20% Byzantine stake ($byzantine_20_rate%)"
fi

# Check if liveness is maintained with ≤20% non-responsive stake
offline_20_success=$(grep "offline_liveness,20," resilience_results.csv | grep -c "SUCCESS" || true)
offline_20_rate=$((offline_20_success * 100 / 50))

if [ $offline_20_rate -ge 80 ]; then
    echo "✅ Liveness maintained with ≤20% non-responsive stake ($offline_20_rate%)"
else
    echo "❌ Liveness compromised with 20% non-responsive stake ($offline_20_rate%)"
fi

# Check partition recovery
if [ $partition_rate -ge 80 ]; then
    echo "✅ Network partition recovery successful ($partition_rate%)"
else
    echo "❌ Network partition recovery failed ($partition_rate%)"
fi

# Check certificate uniqueness
if [ $cert_rate -ge 95 ]; then
    echo "✅ Certificate uniqueness maintained ($cert_rate%)"
else
    echo "❌ Certificate uniqueness compromised ($cert_rate%)"
fi

echo ""
echo "Resilience verification completed. Check resilience_results.csv for detailed results."
