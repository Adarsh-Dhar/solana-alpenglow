#!/bin/bash
# Alpenglow Leader Rotation Verification Script
# This script verifies leader rotation and window management

set -e

echo "=== Alpenglow Leader Rotation Verification ==="
echo "Timestamp: $(date)"
echo "Testing leader rotation and window management"
echo ""

# Create results directory
mkdir -p results/leader_verification
cd results/leader_verification

# Initialize results file
echo "Test_Type,Window_Size,Test_Run,Result,Details" > leader_results.csv

# Test 1: Basic leader rotation
echo "Test 1: Basic leader rotation"

for i in $(seq 1 50); do
    cargo run --release --bin leader_verification -- --test-type rotation --seed $i > rotation_${i}.log 2>&1
    
    if grep -q "Leader rotation successful" rotation_${i}.log; then
        echo "rotation,10,$i,SUCCESS,Valid rotation" >> leader_results.csv
    else
        echo "rotation,10,$i,FAILURE,Invalid rotation" >> leader_results.csv
    fi
done

# Test 2: Window management
echo "Test 2: Window management"

for window_size in 5 10 15 20; do
    echo "Testing window size: $window_size"
    
    for i in $(seq 1 30); do
        cargo run --release --bin leader_verification -- --test-type window --window-size $window_size --seed $i > window_${window_size}_${i}.log 2>&1
        
        if grep -q "Window management successful" window_${window_size}_${i}.log; then
            echo "window,$window_size,$i,SUCCESS,Valid window management" >> leader_results.csv
        else
            echo "window,$window_size,$i,FAILURE,Invalid window management" >> leader_results.csv
        fi
    done
done

# Test 3: BadWindow flag management
echo "Test 3: BadWindow flag management"

for i in $(seq 1 50); do
    cargo run --release --bin leader_verification -- --test-type badwindow --seed $i > badwindow_${i}.log 2>&1
    
    if grep -q "BadWindow management successful" badwindow_${i}.log; then
        echo "badwindow,10,$i,SUCCESS,Valid BadWindow management" >> leader_results.csv
    else
        echo "badwindow,10,$i,FAILURE,Invalid BadWindow management" >> leader_results.csv
    fi
done

# Test 4: Leader failure handling
echo "Test 4: Leader failure handling"

for failure_rate in 10 20 30 40; do
    echo "Testing with $failure_rate% failure rate"
    
    for i in $(seq 1 30); do
        cargo run --release --bin leader_verification -- --test-type failure --failure-rate $failure_rate --seed $i > failure_${failure_rate}pct_${i}.log 2>&1
        
        if grep -q "Failure handling successful" failure_${failure_rate}pct_${i}.log; then
            echo "failure,$failure_rate,$i,SUCCESS,Valid failure handling" >> leader_results.csv
        else
            echo "failure,$failure_rate,$i,FAILURE,Invalid failure handling" >> leader_results.csv
        fi
    done
done

# Test 5: Stake-weighted selection
echo "Test 5: Stake-weighted selection"

for i in $(seq 1 50); do
    cargo run --release --bin leader_verification -- --test-type stake_weighted --seed $i > stake_weighted_${i}.log 2>&1
    
    if grep -q "Stake-weighted selection successful" stake_weighted_${i}.log; then
        echo "stake_weighted,10,$i,SUCCESS,Valid stake-weighted selection" >> leader_results.csv
    else
        echo "stake_weighted,10,$i,FAILURE,Invalid stake-weighted selection" >> leader_results.csv
    fi
done

# Test 6: Window sliding
echo "Test 6: Window sliding"

for i in $(seq 1 30); do
    cargo run --release --bin leader_verification -- --test-type window_sliding --seed $i > window_sliding_${i}.log 2>&1
    
    if grep -q "Window sliding successful" window_sliding_${i}.log; then
        echo "window_sliding,10,$i,SUCCESS,Valid window sliding" >> leader_results.csv
    else
        echo "window_sliding,10,$i,FAILURE,Invalid window sliding" >> leader_results.csv
    fi
done

# Generate summary report
echo ""
echo "=== Leader Rotation Verification Summary ==="

# Analyze results by test type
echo "Results by Test Type:"
for test_type in rotation window badwindow failure stake_weighted window_sliding; do
    count=$(grep "^$test_type," leader_results.csv | wc -l)
    success=$(grep "^$test_type," leader_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  $test_type: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze window size results
echo ""
echo "Window Size Results:"
for window_size in 5 10 15 20; do
    count=$(grep "window,$window_size," leader_results.csv | wc -l)
    success=$(grep "window,$window_size," leader_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  Window size $window_size: $success_rate% success rate ($success/$count)"
    fi
done

# Analyze failure rate results
echo ""
echo "Failure Rate Results:"
for failure_rate in 10 20 30 40; do
    count=$(grep "failure,$failure_rate," leader_results.csv | wc -l)
    success=$(grep "failure,$failure_rate," leader_results.csv | grep -c "SUCCESS" || true)
    
    if [ $count -gt 0 ]; then
        success_rate=$((success * 100 / count))
        echo "  Failure rate $failure_rate%: $success_rate% success rate ($success/$count)"
    fi
done

# Verify critical properties
echo ""
echo "=== Critical Property Verification ==="

# Check if leader rotation works
rotation_success=$(grep "^rotation," leader_results.csv | grep -c "SUCCESS" || true)
rotation_rate=$((rotation_success * 100 / 50))

if [ $rotation_rate -ge 90 ]; then
    echo "✅ Leader rotation works correctly ($rotation_rate%)"
else
    echo "❌ Leader rotation has issues ($rotation_rate%)"
fi

# Check if window management works
window_success=$(grep "^window," leader_results.csv | grep -c "SUCCESS" || true)
window_total=$(grep "^window," leader_results.csv | wc -l)
window_rate=$((window_success * 100 / window_total))

if [ $window_rate -ge 90 ]; then
    echo "✅ Window management works correctly ($window_rate%)"
else
    echo "❌ Window management has issues ($window_rate%)"
fi

# Check if BadWindow management works
badwindow_success=$(grep "^badwindow," leader_results.csv | grep -c "SUCCESS" || true)
badwindow_rate=$((badwindow_success * 100 / 50))

if [ $badwindow_rate -ge 90 ]; then
    echo "✅ BadWindow management works correctly ($badwindow_rate%)"
else
    echo "❌ BadWindow management has issues ($badwindow_rate%)"
fi

# Check if failure handling works
failure_success=$(grep "^failure," leader_results.csv | grep -c "SUCCESS" || true)
failure_total=$(grep "^failure," leader_results.csv | wc -l)
failure_rate=$((failure_success * 100 / failure_total))

if [ $failure_rate -ge 80 ]; then
    echo "✅ Failure handling works correctly ($failure_rate%)"
else
    echo "❌ Failure handling has issues ($failure_rate%)"
fi

# Check if stake-weighted selection works
stake_success=$(grep "^stake_weighted," leader_results.csv | grep -c "SUCCESS" || true)
stake_rate=$((stake_success * 100 / 50))

if [ $stake_rate -ge 90 ]; then
    echo "✅ Stake-weighted selection works correctly ($stake_rate%)"
else
    echo "❌ Stake-weighted selection has issues ($stake_rate%)"
fi

# Check if window sliding works
sliding_success=$(grep "^window_sliding," leader_results.csv | grep -c "SUCCESS" || true)
sliding_rate=$((sliding_success * 100 / 30))

if [ $sliding_rate -ge 90 ]; then
    echo "✅ Window sliding works correctly ($sliding_rate%)"
else
    echo "❌ Window sliding has issues ($sliding_rate%)"
fi

# Generate performance analysis
echo ""
echo "=== Performance Analysis ==="

# Analyze window size performance
echo "Window Size Performance:"
for window_size in 5 10 15 20; do
    avg_time=$(grep "window,$window_size,.*,SUCCESS" leader_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  Window size $window_size: avg time $avg_time"
done

# Analyze failure handling performance
echo ""
echo "Failure Handling Performance:"
for failure_rate in 10 20 30 40; do
    avg_time=$(grep "failure,$failure_rate,.*,SUCCESS" leader_results.csv | awk -F',' '{sum+=$5; count++} END {if(count>0) print sum/count; else print "N/A"}')
    echo "  Failure rate $failure_rate%: avg time $avg_time"
done

# Generate recommendations
echo ""
echo "=== Recommendations ==="

# Check overall success rate
total_tests=$(wc -l < leader_results.csv)
successful_tests=$(grep -c "SUCCESS" leader_results.csv || true)
overall_rate=$((successful_tests * 100 / total_tests))

if [ $overall_rate -ge 90 ]; then
    echo "✅ Overall leader rotation verification successful ($overall_rate%)"
    echo "✅ Leader rotation and window management are working correctly"
    echo "✅ The system can handle leader failures and maintain safety"
else
    echo "⚠️  Overall leader rotation verification needs improvement ($overall_rate%)"
    echo "⚠️  Some aspects of leader rotation may need attention"
fi

# Specific recommendations based on results
if [ $rotation_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing leader rotation algorithm"
fi

if [ $window_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing window management logic"
fi

if [ $badwindow_rate -lt 90 ]; then
    echo "⚠️  Consider reviewing BadWindow flag management"
fi

if [ $failure_rate -lt 80 ]; then
    echo "⚠️  Consider improving failure handling robustness"
fi

echo ""
echo "Leader rotation verification completed. Check leader_results.csv for detailed results."
