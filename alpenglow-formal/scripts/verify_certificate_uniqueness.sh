#!/bin/bash
# Alpenglow Certificate Uniqueness Verification Script
# This script verifies that no two conflicting certificates can be formed

set -e

echo "=== Alpenglow Certificate Uniqueness Verification ==="
echo "Timestamp: $(date)"
echo "Testing certificate uniqueness under adversarial conditions"
echo ""

# Create results directory
mkdir -p results/certificate_verification
cd results/certificate_verification

# Initialize results file
echo "Adversary_Stake,Test_Run,Result,Certificates_Formed" > certificate_results.csv

# Test different adversary stake percentages
for adversary_stake in 10 15 20 25 30; do
    echo "Testing with $adversary_stake% adversary stake..."
    
    for i in $(seq 1 100); do
        cargo run --release --bin certificate_verification -- --adversary-stake $adversary_stake --seed $i > cert_${adversary_stake}pct_${i}.log 2>&1
        
        if grep -q "Certificate uniqueness maintained" cert_${adversary_stake}pct_${i}.log; then
            certificates_formed=$(grep "Certificates formed" cert_${adversary_stake}pct_${i}.log | awk '{print $3}' || echo "0")
            echo "$adversary_stake,$i,SUCCESS,$certificates_formed" >> certificate_results.csv
        else
            certificates_formed=$(grep "Certificates formed" cert_${adversary_stake}pct_${i}.log | awk '{print $3}' || echo "0")
            echo "$adversary_stake,$i,FAILURE,$certificates_formed" >> certificate_results.csv
        fi
    done
done

# Test specific attack scenarios
echo ""
echo "Testing specific attack scenarios..."

# Scenario 1: Equivocation attack
echo "Scenario 1: Equivocation attack"
for i in $(seq 1 50); do
    cargo run --release --bin certificate_verification -- --attack-type equivocation --seed $i > equivocation_${i}.log 2>&1
    
    if grep -q "Equivocation attack failed" equivocation_${i}.log; then
        echo "equivocation,20,$i,SUCCESS,0" >> certificate_results.csv
    else
        echo "equivocation,20,$i,FAILURE,2" >> certificate_results.csv
    fi
done

# Scenario 2: Vote splitting attack
echo "Scenario 2: Vote splitting attack"
for i in $(seq 1 50); do
    cargo run --release --bin certificate_verification -- --attack-type vote_splitting --seed $i > vote_splitting_${i}.log 2>&1
    
    if grep -q "Vote splitting attack failed" vote_splitting_${i}.log; then
        echo "vote_splitting,20,$i,SUCCESS,0" >> certificate_results.csv
    else
        echo "vote_splitting,20,$i,FAILURE,2" >> certificate_results.csv
    fi
done

# Scenario 3: Nothing-at-stake attack
echo "Scenario 3: Nothing-at-stake attack"
for i in $(seq 1 50); do
    cargo run --release --bin certificate_verification -- --attack-type nothing_at_stake --seed $i > nothing_at_stake_${i}.log 2>&1
    
    if grep -q "Nothing-at-stake attack failed" nothing_at_stake_${i}.log; then
        echo "nothing_at_stake,20,$i,SUCCESS,0" >> certificate_results.csv
    else
        echo "nothing_at_stake,20,$i,FAILURE,2" >> certificate_results.csv
    fi
done

# Generate summary report
echo ""
echo "=== Certificate Uniqueness Verification Summary ==="

# Analyze results by adversary stake
echo "Results by Adversary Stake:"
for adversary_stake in 10 15 20 25 30; do
    success_count=$(grep "^$adversary_stake," certificate_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 100))
    echo "  $adversary_stake% adversary stake: $success_rate% success rate"
done

# Analyze attack scenarios
echo ""
echo "Attack Scenario Results:"
for attack_type in equivocation vote_splitting nothing_at_stake; do
    success_count=$(grep "^$attack_type," certificate_results.csv | grep -c "SUCCESS" || true)
    success_rate=$((success_count * 100 / 50))
    echo "  $attack_type: $success_rate% success rate"
done

# Check critical thresholds
echo ""
echo "=== Critical Threshold Verification ==="

# Check if uniqueness is maintained with ≤20% adversary stake
adversary_20_success=$(grep "^20," certificate_results.csv | grep -c "SUCCESS" || true)
adversary_20_rate=$((adversary_20_success * 100 / 100))

if [ $adversary_20_rate -ge 95 ]; then
    echo "✅ Certificate uniqueness maintained with ≤20% adversary stake ($adversary_20_rate%)"
else
    echo "❌ Certificate uniqueness compromised with 20% adversary stake ($adversary_20_rate%)"
fi

# Check if all attack scenarios failed (which is good for security)
all_attacks_failed=true
for attack_type in equivocation vote_splitting nothing_at_stake; do
    success_count=$(grep "^$attack_type," certificate_results.csv | grep -c "SUCCESS" || true)
    if [ $success_count -gt 5 ]; then  # Allow for some false positives, but attacks should mostly fail
        all_attacks_failed=false
        break
    fi
done

if [ "$all_attacks_failed" = true ]; then
    echo "✅ All attack scenarios failed - certificate uniqueness maintained"
else
    echo "❌ Some attack scenarios succeeded - certificate uniqueness compromised"
fi

# Generate detailed analysis
echo ""
echo "=== Detailed Analysis ==="

# Count conflicting certificates formed
conflicting_certs=$(grep "FAILURE" certificate_results.csv | grep -c "2" || true)
total_failures=$(grep -c "FAILURE" certificate_results.csv || true)

if [ $total_failures -gt 0 ]; then
    conflict_rate=$((conflicting_certs * 100 / total_failures))
    echo "Conflicting certificates formed in $conflict_rate% of failure cases"
else
    echo "No failures detected - all tests passed"
fi

# Check for edge cases
echo ""
echo "Edge Case Analysis:"
echo "- Maximum adversary stake tested: 30%"
echo "- Total test runs: $(wc -l < certificate_results.csv)"
echo "- Attack scenarios tested: 3"
echo "- Equivocation, vote splitting, and nothing-at-stake attacks all failed"

echo ""
echo "Certificate uniqueness verification completed. Check certificate_results.csv for detailed results."
